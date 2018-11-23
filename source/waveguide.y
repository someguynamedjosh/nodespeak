%{
#include <cstdio>
#include <iostream>
#include <memory>
#include <vector>

#include "grammar/All.h"
#include "intermediate/Scope.h"

extern "C" int yylex();
extern "C" int yyparse();
extern "C" FILE *yyin;

#define SPN(TYPE, ...) std::shared_ptr<TYPE>{new TYPE(__VA_ARGS__)}
using namespace waveguide::grammar;
template<class T> using sp=std::shared_ptr<T>;
#define dpc std::dynamic_pointer_cast

void yyerror(const char *s);
std::vector<sp<VarDec>> inlineDefs;
sp<StatList> result;
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
	sp<Expression> expression;
	sp<AccessExp> accessexp;
	sp<Statement> statement;
	sp<StatList> statlist;
	sp<DataType> type;
	sp<ExpList> explist;
	sp<OutList> outlist;
	sp<Branch> branch;
	YYSTYPE() { }
	YYSTYPE(YYSTYPE &other) { }
	~YYSTYPE() { }
	YYSTYPE &operator=(YYSTYPE &other) { }
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
	stats { $$ = SPN(StatList, $1); result = $$;}

stats:
	stats stat { $$ = SPN(StatList, $1, $2); }
	| stat { $$ = SPN(StatList, $1); }
	| stats mstat { $$ = SPN(StatList, $1, $2); }
	| mstat { $$ = SPN(StatList, $1); }

mstat:
	vardec ';' { $$ = $1; }
	| RETURN exp ';' { 
		$$ = SPN(StatList, 
			SPN(AssignStat, 
				SPN(AccessExp, SPN(IdentifierExp, "return")), 
				$2
			), 
			SPN(ReturnStat)
		); 
	}
	| accessexp A exp ';' { 
		$$ = SPN(StatList);
		for(auto v : inlineDefs) 
			$$->append(v); 
		inlineDefs.clear(); 
		$$->append(SPN(AssignStat, $1, $3));
	} 
	| IDENTIFIER '(' explist ')' ':' '(' outlist ')' ';' { 
		$$ = new StatList()); 
		for(auto v : inlineDefs) 
			$$->append(v); 
		inlineDefs.clear(); 
		$$->append(new FuncCallStat(new FuncCall($1, $3, $7)));
	}
	| IDENTIFIER '(' explist ')' ';' { 
		$$ = SPN(StatList); 
		$$->append(SPN(FuncCallStat, SPN(FuncCall, $1, $3, SPN(OutList))));
	}

vardec:
	type IDENTIFIER { $$ = SPN(StatList, SPN(VarDec, $1, $2)); }
	| type IDENTIFIER A exp { 
		$$ = SPN(StatList, SPN(VarDec, $1, $2)); 
		for(auto v : inlineDefs) 
			$$->append(v);
		inlineDefs.clear(); 
		$$->append(SPN(AssignStat, SPN(AccessExp, SPN(IdentifierExp, $2)), $4));
	}
	| vardec ',' IDENTIFIER { 
		$$ = SPN(StatList, $1, 
			SPN(VarDec, 
				dpc<VarDec>($1->getStatements()[0])->getType(), 
				$3
			)
		);
	}
	| vardec ',' IDENTIFIER A exp { 
		$$ = SPN(StatList, 
			$1, 
			SPN(VarDec, 
				dpc<VarDec>($1->getStatements()[0])->getType(), 
				$3
			)
		);
		$$->append(SPN(AssignStat, SPN(AccessExp, SPN(IdentifierExp, $3)), $5));
	} 

indec:
	type IDENTIFIER { $$ = SPN(StatList, SPN(VarDec, $1, $2)); }
	| indec ',' indec { $$ = SPN(StatList, $1, $3); }

indec2:
	indec { $$ = $1; }
	| %empty { $$ = SPN(StatList); }

outdec:
	type IDENTIFIER { $$ = SPN(StatList, SPN(VarDec, $1, $2)); }
	| type RETURN { $$ = SPN(StatList, SPN(VarDec, $1, "return")); }
	| outdec ',' outdec { $$ = SPN(StatList, $1, $3); }

explist:
	exp { $$ = SPN(ExpList, $1); }
	| explist ',' exp { $$ = SPN(ExpList, $1, $3); }

outlist:
	RETURN { $$ = SPN(OutList, SPN(RetOut)); }
	| NONE { $$ = SPN(OutList, SPN(NoneOut)); }
	| accessexp { $$ = SPN(OutList, SPN(VarAccessOut, $1)); }
	| type IDENTIFIER { 
		$$ = SPN(OutList, SPN(VarAccessOut, SPN(IdentifierExp, $2))); 
		inlineDefs.push_back(SPN(VarDec, $1, $2)); }
	| outlist ',' outlist { $$ = SPN(OutList, $1, $3); }

stat:
	IDENTIFIER '(' indec2 ')' '{' stats '}' { $$ = SPN(FuncDec, $1, $3, SPN(StatList), $6); }
	| IDENTIFIER '(' indec2 ')' ':' IDENTIFIER '{' stats '}' { 
		$$ = SPN(FuncDec, $1, $3, SPN(StatList, SPN(VarDec, SPN(NamedDataType, $6), "return")), $8); }
	| IDENTIFIER '(' indec2 ')' ':' '(' outdec ')' '{' stats '}' { $$ = SPN(FuncDec, $1, $3, $7, $10); }
	| branch { $$ = $1; }
	| branch ELSE '{' stats '}' { $1->addElse($4); $$ = $1; } 
	| FOR '(' type IDENTIFIER OF explist ')' '{' stats '}' { $$ = SPN(ForLoop, SPN(VarDec, $3, $4), $6, $9); }
	| WHILE '(' exp ')' '{' stats '}' { $$ = SPN(WhileLoop, $3, $6); }
	| RETURN ';' { $$ = SPN(ReturnStat); }

branch:
	IF '(' exp ')' '{' stats '}' { $$ = SPN(Branch, $3, $6); }
	| branch ELIF '(' exp ')' '{' stats '}' { $$ = $1; $1->addElif(SPN(Branch, $4, $7)); }

type:
	IDENTIFIER { $$ = SPN(NamedDataType, $1); }
	| type '[' exp ']' { $$ = SPN(ArrayDataType, $1, $3); }

accessexp:
	IDENTIFIER { $$ = SPN(AccessExp, SPN(IdentifierExp, $1)); }
	| accessexp '[' exp ']' { $$ = $1; $$->addIndexAccessor($3); }

exp:
	exp '+' exp { $$ = SPN(AddExp, $1, $3); }
	| exp '-' exp { $$ = SPN(AddExp, $1, SPN(MulExp, $3, SPN(IntExp, -1))); }
	| exp '*' exp { $$ = SPN(MulExp, $1, $3); }
	| exp '/' exp { $$ = SPN(MulExp, $1, SPN(RecipExp, $3)); }
	| exp '%' exp { $$ = SPN(ModExp, $1, $3); }
	| '(' exp ')' { $$ = $2; }
	| exp EQ exp { $$ = SPN(EqExp, $1, $3); }
	| exp NEQ exp { $$ = SPN(NeqExp, $1, $3); }
	| exp LTE exp { $$ = SPN(LteExp, $1, $3); }
	| exp GTE exp { $$ = SPN(GteExp, $1, $3); }
	| exp LT exp { $$ = SPN(LtExp, $1, $3); }
	| exp GT exp { $$ = SPN(GtExp, $1, $3); }
	| exp AND exp { $$ = SPN(AndExp, $1, $3); }
	| exp OR exp { $$ = SPN(OrExp, $1, $3); }
	| exp XOR exp { $$ = SPN(XorExp, $1, $3); }
	| exp BAND exp { $$ = SPN(BandExp, $1, $3); }
	| exp BOR exp { $$ = SPN(BorExp, $1, $3); }
	| exp BXOR exp { $$ = SPN(BxorExp, $1, $3); }
	| accessexp { $$ = $1; }
	| INT { $$ = SPN(IntExp, $1); } 
	| FLOAT { $$ = SPN(FloatExp, $1); }
	| IDENTIFIER '(' explist ')' { $$ = SPN(FuncCall, $1, $3, SPN(OutList, SPN(RetOut))); } 
	| IDENTIFIER '(' explist ')' ':' '(' outlist ')' { $$ = SPN(FuncCall, $1, $3, $7); }
	| '[' explist ']' { $$ = SPN(ArrayLiteral, $2); }
	| '{' exp ',' exp '}' { $$ = SPN(Range, $2, $4); }
	| '{' exp ',' exp ',' exp '}' { $$ = SPN(Range, $2, $4, $6); }

%%

int main(int, char**) {
	FILE *input = fopen("sample.wg", "r");
	if(!input) {
		std::cerr << "Error opening sample.wg" << std::endl;
		return -1;
	}
	yyin = input;
	do { 
		yyparse();
	} while (!feof(yyin));
	sp<waveguide::intermediate::Scope> root = waveguide::convert::parseSyntaxTree(result);
	//Com::interpret(root);
}

