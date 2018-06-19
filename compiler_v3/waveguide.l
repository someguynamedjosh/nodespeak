%{
#include <iostream>
#include "tokens.h"
#include "waveguide.tab.h"
using namespace std;
#define YY_DECL extern "C" int yylex()
%}
%option yylineno
%%
[ \t\n]  ;
\#.*\n ;
if { return IF; }
elif { return ELIF; }
else { return ELSE; }
for { return FOR; }
of { return OF; }
while { return WHILE; }
do { return DO; }
typedef { return TYPEDEF; }
break { return BREAK; }
return { return RETURN; }
none { return NONE; }
def { return DEF; }
\-\= { return ASUB; }
\+\= { return AADD; }
\*\= { return AMUL; }
\/\= { return ADIV; }
\%\= { return AMOD; }
\@\= { return AXOR; }
\/\@\= { return ABXOR; }
\|\= { return AOR; }
\/\|\= { return ABOR; }
\&\= { return AAND; }
\/\&\= { return ABAND; }
\=\! { return ANOT; }
\=\/\! { return ABNOT; }
\/\@ { return BXOR; }
\/\| { return BOR; }
\/\& { return BAND; }
\=\= { return EQ; }
\!\= { return NEQ; }
\<\= { return LTE; }
\>\= { return GTE; }
\< { return LT; }
\> { return GT; }
\= { return A; }
((([0-9]+)?\.[0-9]+)|([0-9]+\.))(([eE][+-]?[0-9]+)|([0-9]+[eE][+-]?[0-9]+))?  { yylval.fval = atof(yytext); return FLOAT; }
[0-9]+  { yylval.ival = atoi(yytext); return INT; }
[a-zA-Z_][a-zA-Z0-9_]*  { yylval.sval = strdup(yytext); return IDENTIFIER; }
[\.\:\;\,\(\)\{\}\[\]] { return yytext[0]; }
[\%\/\*\+\-\=\@\|\&\=] { return yytext[0]; }
. ;

%%

void yyerror(const char *s) {
	cerr << "A parse error has occured: " << s << endl;
	cerr << "Occured on line " << yylineno << endl;
	exit(-1);
}
