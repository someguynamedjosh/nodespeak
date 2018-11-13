%{
#include<cstdio>
#include<iostream>
#include<vector>
#include "tokens.h"
#include "scope.h"
#include "interpreter.h"
using namespace std;

extern "C" int yylex();
extern "C" int yyparse();
extern "C" FILE *yyin;

void yyerror(const char *s);
vector<VarDec*> inlineDefs;
StatList *result;
%}

%glr-parser
%error-verbose
%expect-rr 3
%locations

%union {
	int ival;
	float fval;
	char cval;
	char *sval;
	Expression *expression;
	AccessExp *accessexp;
	Statement *statement;
	StatList *statlist;
	Type *type;
	ExpList *explist;
	OutList *outlist;
	Branch *branch;
}

%token <ival> INT
%token <fval> FLOAT
%token <sval> IDENTIFIER
%token IF ELIF ELSE FOR WHILE DO
%token BREAK RETURN A OF TYPEDEF NONE DEF
%token AND OR XOR NOT BAND BOR BXOR BNOT
%token AAND AOR AXOR ABAND ABOR ABXOR
%token AADD ASUB AMUL ADIV AMOD
%token GT LT GTE LTE EQ NEQ

%type <expression> exp
%type <statement> stat
%type <accessexp> accessexp
%type <statlist> stats
%type <statlist> root
%type <type> type
%type <statlist> vardec
%type <statlist> mstat
%type <statlist> indec
%type <statlist> indec2
%type <statlist> outdec
%type <explist> explist
%type <outlist> outlist
%type <branch> branch

%left A ASUB AADD AMUL ADIV AMOD AXOR ABXOR AOR ABOR AAND ABAND
%left ','
%left OR XOR AND BOR BXOR BAND
%left EQ NEQ LTE GTE LT GT
%left '+' '-' '*' '/' '%'
%left '.' '['
%%

root:
	stats { $$ = new StatList($1); result = $$; }

stats:
	stats stat { $$ = new StatList($1, $2); }
	| stat { $$ = new StatList($1); }
	| stats mstat { $$ = new StatList($1, $2); }
	| mstat { $$ = new StatList($1); }

mstat:
	vardec ';' { $$ = $1; }
	| RETURN exp ';' { 
		$$ = new StatList(new AssignStat(new AccessExp(new IdentifierExp("return")), $2), new ReturnStat()); }
	| accessexp A exp ';' { 
		$$ = new StatList(); 
		for(VarDec* v : inlineDefs) 
			$$->append(v); 
		inlineDefs.clear(); 
		$$->append(new AssignStat($1, $3)); } 
	| IDENTIFIER '(' explist ')' ':' '(' outlist ')' ';' { 
		$$ = new StatList(); 
		for(VarDec* v : inlineDefs) 
			$$->append(v); 
		inlineDefs.clear(); 
		$$->append(new FuncCallStat(new FuncCall($1, $3, $7))); }
	| IDENTIFIER '(' explist ')' ';' { 
		$$ = new StatList(); 
		$$->append(new FuncCallStat(new FuncCall($1, $3, new OutList()))); }

vardec:
	type IDENTIFIER { $$ = new StatList(new VarDec($1, $2)); }
	| type IDENTIFIER A exp { 
		$$ = new StatList(new VarDec($1, $2)); 
		for(VarDec* v : inlineDefs) 
			$$->append(v); inlineDefs.clear(); 
		$$->append(new AssignStat(new AccessExp(new IdentifierExp($2)), $4)); }
	| vardec ',' IDENTIFIER { $$ = new StatList($1, new VarDec(((VarDec*) $1->getStatements()[0])->getType(), $3)); }
	| vardec ',' IDENTIFIER A exp { 
		$$ = new StatList($1, new VarDec(((VarDec*) $1->getStatements()[0])->getType(), $3));
		$$->append(new AssignStat(new AccessExp(new IdentifierExp($3)), $5)); } 

indec:
	type IDENTIFIER { $$ = new StatList(new VarDec($1, $2)); }
	| indec ',' indec { $$ = new StatList($1, $3); }

indec2:
	indec { $$ = $1; }
	| %empty { $$ = new StatList(); }

outdec:
	type IDENTIFIER { $$ = new StatList(new VarDec($1, $2)); }
	| type RETURN { $$ = new StatList(new VarDec($1, "return")); }
	| outdec ',' outdec { $$ = new StatList($1, $3); }

explist:
	exp { $$ = new ExpList($1); }
	| explist ',' exp { $$ = new ExpList($1, $3); }

outlist:
	RETURN { $$ = new OutList(new RetOut()); }
	| NONE { $$ = new OutList(new NoneOut()); }
	| accessexp { $$ = new OutList(new VarAccessOut($1)); }
	| type IDENTIFIER { 
		$$ = new OutList(new VarAccessOut(new IdentifierExp($2))); 
		inlineDefs.push_back(new VarDec($1, $2)); }
	| outlist ',' outlist { $$ = new OutList($1, $3); }

stat:
	IDENTIFIER '(' indec2 ')' '{' stats '}' { $$ = new FuncDec($1, $3, new StatList(), $6); }
	| IDENTIFIER '(' indec2 ')' ':' IDENTIFIER '{' stats '}' { 
		$$ = new FuncDec($1, $3, new StatList(new VarDec(new TypeName($6), "return")), $8); }
	| IDENTIFIER '(' indec2 ')' ':' '(' outdec ')' '{' stats '}' { $$ = new FuncDec($1, $3, $7, $10); }
	| branch { $$ = $1; }
	| branch ELSE '{' stats '}' { $1->addElse($4); $$ = $1; } 
	| FOR '(' type IDENTIFIER OF explist ')' '{' stats '}' { $$ = new ForLoop(new VarDec($3, $4), $6, $9); }
	| WHILE '(' exp ')' '{' stats '}' { $$ = new WhileLoop($3, $6); }
	| RETURN ';' { $$ = new ReturnStat(); }

branch:
	IF '(' exp ')' '{' stats '}' { $$ = new Branch($3, $6); }
	| branch ELIF '(' exp ')' '{' stats '}' { $$ = $1; $1->addElif(new Branch($4, $7)); }

type:
	IDENTIFIER { $$ = new TypeName($1); }
	| type '[' exp ']' { $$ = new ArrayType($1, $3); }

accessexp:
	IDENTIFIER { $$ = new AccessExp(new IdentifierExp($1)); }
	| accessexp '[' exp ']' { $$ = $1; $$->addIndexAccessor($3); }

exp:
	exp '+' exp { $$ = new AddExp($1, $3); }
	| exp '-' exp { $$ = new AddExp($1, new MulExp($3, new IntExp(-1))); }
	| exp '*' exp { $$ = new MulExp($1, $3); }
	| exp '/' exp { $$ = new MulExp($1, new RecipExp($3)); }
	| exp '%' exp { $$ = new ModExp($1, $3); }
	| '(' exp ')' { $$ = $2; }
	| exp EQ exp { $$ = new EqExp($1, $3); }
	| exp NEQ exp { $$ = new NeqExp($1, $3); }
	| exp LTE exp { $$ = new LteExp($1, $3); }
	| exp GTE exp { $$ = new GteExp($1, $3); }
	| exp LT exp { $$ = new LtExp($1, $3); }
	| exp GT exp { $$ = new GtExp($1, $3); }
	| exp AND exp { $$ = new AndExp($1, $3); }
	| exp OR exp { $$ = new OrExp($1, $3); }
	| exp XOR exp { $$ = new XorExp($1, $3); }
	| exp BAND exp { $$ = new BandExp($1, $3); }
	| exp BOR exp { $$ = new BorExp($1, $3); }
	| exp BXOR exp { $$ = new BxorExp($1, $3); }
	| accessexp { $$ = $1; }
	| INT { $$ = new IntExp($1); } 
	| FLOAT { $$ = new FloatExp($1); }
	| IDENTIFIER '(' explist ')' { $$ = new FuncCall($1, $3, new OutList(new RetOut())); } 
	| IDENTIFIER '(' explist ')' ':' '(' outlist ')' { $$ = new FuncCall($1, $3, $7); }
	| '[' explist ']' { $$ = new ArrayLiteral($2); }
	| '{' exp ',' exp '}' { $$ = new Range($2, $4); }
	| '{' exp ',' exp ',' exp '}' { $$ = new Range($2, $4, $6); }

%%

int main(int, char**) {
	FILE *input = fopen("sample.wg", "r");
	if(!input) {
		cerr << "Error opening sample.wg" << endl;
		return -1;
	}
	yyin = input;
	do { 
		yyparse();
	} while (!feof(yyin));
	Com::Scope *root = Com::parseSyntaxTree(result);
	Com::interpret(root);
}
