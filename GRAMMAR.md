# Parsing grammar
What follows is the SBL parsing grammar. This is not BNF, but close to it.

```
source = fundef*

top_level = import
          | foreign
          | fundef

import = 'import' string

foreign = 'foreign' '{' foreign_def* '}'

foreign_def = ident ident '[' ident* ']'

fundef = <ident> block

block = '{' line* '}'

line = action
     | branch
     | loop

action = '.' ( ident | nil )
       | item

branch = <br> action* block
       | <br> action* block branch_tail

branch_tail = <elbr> action* block branch_tail
            | <el> block

loop = <loop> action* block

item = <ident>
     | <num>
     | <sym>
```

# Available tokens
These are the tokens that are recognized by the tokenizer.

```
comment = '#' .+ $

num = '-'? [1-9][0-9]*
    | '-'? '0' [xX] [0-9a-fA-F]+
    | '-'? '0' [bB] [01]+

ident = [A-z_!$%^&*-+/]+

nil = '@'

dot = '.'

br = 'br'

elbr = 'elbr'

el = 'el'

lbrace = '{'

rbrace = '}'

lbrack = '['

rbrack = ']'

string = '"' ( escape | non-EOF-dquote-newline )* '"'

```
