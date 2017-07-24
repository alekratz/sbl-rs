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

action = '.' ident*
       | item*

branch = <br> block
       | <br> block <el> block

loop = <loop> block

item = <ident>
     | <num>
     | <sym>
```

# Available tokens
These are the tokens that are recognized by the tokenizer.

```
comment = '#' .+ $

num = [1-9][0-9]*
    | '0' [xX] [0-9a-fA-F]+
    | '0' [bB] [01]+

ident = [A-z_!@$%^&*-+/]+

dot = '.'

br = 'br'

el = 'el'

lbrace = '{'

rbrace = '}'

lbrack = '['

rbrack = ']'

string = '"' ( escape | non-EOF-dquote-newline )* '"'

```
