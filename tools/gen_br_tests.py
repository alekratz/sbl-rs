#!/usr/bin/env python3
# gen_br_tests.py
# Generates branching integration tests for copy-pasting into Rust.
#  * They follow a pattern
#  * More tests are better
#  * They'd take a long time to comprehensively cover by hand
from typing import *
import abc
import sys
from itertools import chain, combinations, product


################################################################################
# Utility functions and errors
################################################################################

class EmptyStackError(Exception):
    '''Gets raised during control flow diagram creation, if a control flow would end with an empty stack.'''
    def __init__(self, sender, *args, **kwargs):
        super().__init__(self, *args, **kwargs)
        self.sender = sender


def powerset(iterable):
    '''Powerset function, stolen from python.org'''
    s = list(iterable)
    return chain.from_iterable(combinations(s, r) for r in range(len(s)+1))


class Emit(metaclass=abc.ABCMeta):
    '''Something that emits SBL code.'''
    @abc.abstractmethod
    def emit(self):
        '''Abstract emit method. Indicates that a class will emit SBL code.'''

    def ensure_tree(self):
        '''
        Ensures that the entire tree of this object is made up of emitters.
        This is not required to be implemented.
        '''
        return self

    def _ensure_tree(self, tree):
        return [v.ensure_tree() if isinstance(v, Emit) else Push(v) for v in tree]

    @staticmethod
    def dispatch(e):
        '''Calls and returns emit() on the given object'''
        assert isinstance(e, Emit)
        return e.emit()


class Flow(metaclass=abc.ABCMeta):
    '''Something that contributes to a flow graph.'''
    @abc.abstractmethod
    def flow(self):
        '''
        Makes the control flow graph for this object.
        :returns: the stack that is expected to be created.'''

    def is_valid(self):
        if isinstance(self, Emit):
            self.ensure_tree()
        try:
            self.flow()
        except EmptyStackError:
            return False
        else:
            return True

    @staticmethod
    def dispatch(f):
        '''Calls and returns flow() on the given object'''
        assert isinstance(f, Flow)
        return f.flow()

    @staticmethod
    def flow_list(ls: list):
        return sum(map(Flow.dispatch, ls), [])


################################################################################
# Statements
################################################################################

class Push(Emit, Flow):
    '''
    A single push statement. Its value can be any valid SBL value.
    Booleans, strings, and integers definitely work.
    Characters do not.
    '''
    def __init__(self, val):
        self.val = val

    def emit(self):
        '''Generates the SBL code for this value.'''
        v = self.val
        if isinstance(v, str):
            return '"{}"'.format(v)
        elif isinstance(v, bool):
            return 'T' if v else 'F'
        elif v is None:
            return '@'
        else:
            return str(v)

    def remit(self):
        v = self.val
        if isinstance(v, str):
            return 'BCVal::String(String::from(r#"{}"#))'.format(v)
        elif isinstance(v, bool):
            return 'BCVal::Bool({})'.format('true' if v else 'false')
        elif v is None:
            return 'BCVal::Nil'
        else:
            return 'BCVal::Int({})'.format(v)

    def is_true(self) -> bool:
        return self.val != False and self.val is not None

    def flow(self):
        return [self]

    def __repr__(self):
        return "({})".format(self.emit())


class ElStmt(Emit):
    def __init__(self, body: Sequence[Emit]):
        self.body = body

    def emit(self):
        '''Generates tthe SBL code for this statement.'''
        return 'el {{ {body} }}'.format(body=' '.join(map(Emit.dispatch, self.body)))

    def ensure_tree(self):
        self.body = self._ensure_tree(self.body)
        return self


class ElBrStmt(Emit):
    def __init__(self, cond: Sequence[Emit], body: Sequence[Emit]):
        self.cond = cond
        self.body = body

    def emit(self):
        '''Generates tthe SBL code for this statement.'''
        return 'elbr {cond} {{ {body} }}'.format(
                cond=map(' '.join(Emit.dispatch, self.cond)),
                body=map(' '.join(Emit.dispatch, self.body)))

    def ensure_tree(self):
        self.cond = self._ensure_tree(self.cond)
        self.body = self._ensure_tree(self.body)
        return self


class BrStmt(Emit, Flow):
    def __init__(self, pre: Sequence[Emit], cond: Sequence[Push], body: Sequence[Emit], elbr: Sequence[ElBrStmt], el: Optional[ElStmt], post: Sequence[Emit]):
        assert isinstance(el, ElStmt) or el is None, 'el must be ElStmt or None, instead got {}'.format(type(el))
        self.pre = pre
        self.cond = cond
        self.body = body
        self.elbr = elbr
        self.el = el
        self.post = post

    def emit(self):
        '''Generates tthe SBL code for this statement.'''
        return "{pre} br {cond} {{ {body} }} {elbr} {el} {post}".format(
                pre=' '.join(map(Emit.dispatch, self.pre)),
                cond=' '.join(map(Emit.dispatch, self.cond)),
                body=' '.join(map(Emit.dispatch, self.body)),
                elbr=' '.join(map(Emit.dispatch, self.elbr)),
                el = self.el.emit() if self.el else '',
                post=' '.join(map(Emit.dispatch, self.post))).strip().replace('  ', ' ')

    def remit(self):
        '''Generates the Rust code for this statement'''
        return 'state_test!(r#"main {{ {sbl} }}"#, vec![{stack}]);'.format(
                sbl=self.emit(), stack=', '.join(map(Push.remit, self.flow())))

    def ensure_tree(self):
        self.pre = self._ensure_tree(self.pre)
        self.cond = self._ensure_tree(self.cond)
        self.body = self._ensure_tree(self.body)
        self.elbr = self._ensure_tree(self.elbr)
        self.el = ElStmt(self._ensure_tree(self.el.body)) if self.el is not None else None
        self.post = self._ensure_tree(self.post)
        return self

    def flow(self):
        def check_stack(st, sender=self):
            if not st: raise EmptyStackError(sender)
        flow = Flow.flow_list
        st = []
        st += flow(self.pre)
        st += flow(self.cond)
        check_stack(st)
        top = st.pop()
        assert isinstance(top, Push), "expected stack item to be Push but instead it was {}".format(type(top))
        if top.is_true():
            st += flow(self.body)
        else:
            done = False
            for elbr in self.elbr:
                st += flow(elbr.cond)
                check_stack()
                top = st.pop()
                if top:
                    st += flow(elbr.body)
                    done = True
                    break
            if not done and self.el is not None:
                st += flow(self.el.body)
        st += flow(self.post)
        return st

    def __repr__(self):
        return "pre={} cond={} body={} elbr={} el={} post={}".format(
                self.pre, self.cond, self.body, self.elbr, self.el, self.post)

################################################################################
# Permutation functions
################################################################################

def permute_el(body=[]):
    return map(lambda a: ElStmt(*a), product(powerset(body)))


def permute_elbr(cond=[], body=[]):
    return map(lambda a: ElBrStmt(*a),
               product(powerset(cond), powerset(body)))


def permute_br(pre=[], cond=[], body=[], elbr=[], el=[], post=[], filt=lambda _:True):
    '''Permutes through all solo branch possibilities with a handful of arguments.'''
    return filter(filt, filter(Flow.is_valid, map(lambda a: BrStmt(*a),
               product(powerset(pre),
                       powerset(cond),
                       powerset(body),
                       powerset(elbr),
                       map(lambda t: t[0] if len(t) > 0 else None, powerset(el)),
                       powerset(post)))))


def permute_el(body=[], filt=lambda _:True):
    '''Permutes through all solo branch possibilities with a handful of arguments.'''
    return filter(filt, map(lambda a: ElStmt(a), powerset(body)))

################################################################################
# Code generation functions
################################################################################

def generate_br_solo():
    '''Generate solo br statement tests'''
    inner = list(permute_br(pre=[5678],
                       cond=[True, False],
                       body=[2222],
                       post=[8765],
                       filt=lambda x: bool(x.body)))
    outer = list(permute_br(cond=[1111, True, False]))

    for obr in outer:
        for ibr in inner:
            obr.body = [ibr]
            print(obr.remit())


def generate_br_el():
    '''Generate br { ... } el { ... } statement tests'''
    br_filt = lambda x: bool(x.body) and bool(x.el)
    el_filt = lambda x: bool(x.body)
    inner = list(permute_br(pre=[5678],
                       cond=[True, False],
                       body=[2222],
                       post=[8765],
                       el=permute_el(body=[3333], filt=el_filt),
                       filt=br_filt))
    outer = list(permute_br(cond=[1111, True, False]))

    for obr in outer:
        for ibr in inner:
            obr.body = [ibr]
            print(obr.remit())

################################################################################
# Main function
################################################################################

def main():
    '''Program entry point'''
    argv = sys.argv
    unimp = lambda: print("UNIMPLEMENTED, PLEASE CALL 1-800-R-U-SLAPPIN FOR ASSISTANCE")
    genmap = {
        'br_solo': generate_br_solo,
        'br_el': generate_br_el,
        'br_elbr': unimp,
        'br_elbr_el': unimp,
    }
    def show_usage():
        print("usage: {} SUITE".format(argv[0]), file=sys.stderr)
        print("where SUITE is one of the following:", file=sys.stderr)
        print('    * ' + '\n    * '.join(genmap.keys()), file=sys.stderr)
        sys.exit(1)

    if len(argv) != 2:
        show_usage()


    which = argv[1]
    if which not in genmap:
        show_usage()
    genmap[which]()

if __name__ == '__main__':
    main()
