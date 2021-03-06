" Language: SBL

if exists("b:current_syntax")
    finish
endif

" Functions and imports
syn keyword sblImportKeyword import nextgroup=sblString
syn match sblFunction '[a-zA-Z_!$%^&|*\-+/=<>]\+' nextgroup=sblBlock skipwhite

" Foreign block
syn keyword sblForeign foreign nextgroup=sblForeignLib skipwhite
syn match sblForeignLib "\"[^\"]*\"" contains=sblEscapes nextgroup=sblForeignBlock skipwhite
syn region sblForeignBlock start='{' end='}' fold contains=sblForeignKeywords,sblForeignFunction,sblComment
syn keyword sblForeignKeywords containedin=sblForeignBlock int string stack
syn match sblForeignFunction '[a-zA-Z_][a-zA-Z_0-9]*' containedin=sblForeignBlock

" Code blocks
syn region sblBlock start='{' end='}' fold contains=sblBlock,@sblBody
syn cluster sblBody contains=sblComment,sblIdent,sblBake,@sblLiteral,@sblKeywords
syn cluster sblKeywords contains=sblCond,sblLoop,sblOps,sblNil,sblPop
syn keyword sblCond contained br elbr el
syn keyword sblLoop contained loop
syn keyword sblOps contained < > <= >= == !=
syn keyword sblBake contained bake nextgroup=sblBlock skipwhite
syn match sblNil '@' contained
syn match sblIdent '[a-zA-Z_!$%^&|*+/=<>]\+' contained
syn match sblPop /\./ contained nextgroup=sblNil,sblNumber,sblIdent

" Literals and escapes
syn cluster sblLiteral contains=sblString,sblChar,sblNumber,sblBool
syn keyword sblEscapes contained \\n \\r \\s \\0 \\" \\'
syn region sblString start=/"/ skip=/\\"/ end=/"/ contains=sblEscapes
syn match sblChar "'."
syn match sblChar "'\\." contains=sblEscapes
syn match sblNumber "-\?\(0[xXbBo]\)\?[0-9]\+"
syn keyword sblBool T F

" Comments
syn match sblTodo contained "TODO" "FIXME" "XXX" "NOTE"
syn match sblComment ";.*$" contains=sblTodo
syn region sblComment start=";!" end="!;" contains=sblTodo

let b:current_syntax = "sbl"

" Comments
hi def link sblTodo             Todo
hi def link sblComment          Comment

" Literals
hi def link sblForeignLib       String
hi def link sblString           String
hi def link sblChar             Character
hi def link sblEscapes          SpecialChar
hi def link sblNumber           Number
hi def link sblBool             Boolean

" Names
hi def link sblFunction         Function
hi def link sblForeignFunction  Function
"hi def link sblIdent            Identifier

" Keywords
hi def link sblForeign          Keyword
hi def link sblImportKeyword    Include
hi def link sblCond             Conditional
hi def link sblLoop             Repeat
hi def link sblOps              Operator
hi def link sblBake             PreProc
hi def link sblNil              Keyword

" Types
hi def link sblForeignKeywords  Type

" Statements
hi def link sblPop              Statement
