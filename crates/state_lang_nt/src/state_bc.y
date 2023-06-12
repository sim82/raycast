%start File
%avoid_insert "INT"
%%
File -> Result<Vec<Toplevel>, Box<dyn Error>>:
	Toplevel { Ok(vec![$1?]) }
	| File Toplevel { flatten( $1, $2 ) }
	;

Toplevel -> Result<Toplevel, Box<dyn Error>>: 
	'states' 'IDENTIFIER' '{' StatesBody '}' {
		Ok(Toplevel::States{name: $2?.span(), elements: $4?})
	}
	| 'enum' 'IDENTIFIER' '{' EnumBody '}' {
		Ok(Toplevel::Enum{name: $2?.span(), elements: $4?})
	} 
	| 'spawn' 'IDENTIFIER' '{' SpawnBody '}' {
		Ok(Toplevel::Spawn{ name: $2?.span(), elements: $4? })
	}
	| 'function' 'IDENTIFIER' '{' WordList '}' {
		Ok(Toplevel::Function{ name: $2?.span(), body: $4?  })
	}
	;


StatesBody -> Result<Vec<StateElement>, Box<dyn Error>>:
	StateElement { Ok(vec![$1?])}
	| StatesBody StateElement { flatten($1, $2) }  
	;

StateElement -> Result<StateElement,Box<dyn Error>>:
	'state' 'IDENTIFIER' '::' 'IDENTIFIER' ',' Bool ',' Expr ',' FunctionRef ',' FunctionRef ',' 'IDENTIFIER' {
		Ok(StateElement::State { sprite: ($2?.span(),$4?.span()), directional: $6?, timeout: $8?, think: $10?, action: $12?, next: $14?.span() })
	}
	| 'IDENTIFIER' ':' {
		Ok(StateElement::Label ( $1?.span() ))
	}
	;


EnumBody -> Result<Vec<Span>,Box<dyn Error>>:
	'IDENTIFIER' { Ok(vec![$1?.span()])}
	| EnumBody ',' 'IDENTIFIER' { flatten($1,Ok($3?.span()))	}
	;
	
	
SpawnBody -> Result<Vec<SpawnElement>,Box<dyn Error>>:
	SpawnElement { Ok(vec![$1?])}
	| SpawnBody SpawnElement { flatten($1,$2) }
	;

SpawnElement -> Result<SpawnElement, Box<dyn Error>>:
	'directional' Expr ',' 'IDENTIFIER' ',' 'IDENTIFIER' {
		Ok(SpawnElement { directional: true, id: $2?, state: $4?.span(), drop: $6?.span() })
	}
	| 'undirectional' Expr ',' 'IDENTIFIER' ',' 'IDENTIFIER' {
		Ok(SpawnElement { directional: false, id: $2?, state: $4?.span(), drop: $6?.span() })
	}
	;

FunctionRef -> Result<FunctionRef, Box<dyn Error>>:
	'IDENTIFIER' {Ok(FunctionRef::Name($1?.span()))}
	| '{' WordList '}' {Ok(FunctionRef::Inline($2?))}
	;

WordList -> Result<Vec<Word>, Box<dyn Error>>:
	Word { Ok(vec![$1?]) }
	| WordList Word { flatten($1, $2) }
	;
	
Word -> Result<Word, Box<dyn Error>>:
	TypedIntExpr { Ok(Word::Push($1?)) }
	| 'IDENTIFIER' '::' 'IDENTIFIER' {Ok(Word::PushEnum($1?.span(),$3?.span()))}
	| 'trap' { Ok(Word::Trap )}
	| 'not' { Ok(Word::Not)}
	| 'if' WordList 'then' { Ok(Word::If($2?))}
	| 'STATE_LABEL' { Ok(Word::PushStateLabel($1?.span()))}
	| 'gostate' { Ok(Word::GoState) }
	| 'stop' { Ok(Word::Stop) }
	| 'add' { Ok(Word::Add)}
	| 'call' { Ok(Word::Call)}
	| '[' WordList ']' { Ok(Word::WordList($2?))}
	;

TypedIntExpr -> Result<TypedInt, Box<dyn Error>>:
	Expr TypeName {
		match $2? {
			TypeName::U8 => Ok(TypedInt::U8($1? as u8)),
			TypeName::I32 => Ok(TypedInt::I32($1? as i32)),
		}
	}
	| Expr {
		Ok(TypedInt::U8($1? as u8))
	}
	;

TypeName -> Result<TypeName, Box<dyn Error>>: 
	'u8' { Ok(TypeName::U8) }
	| 'i32' { Ok(TypeName::I32) }
	;
	
Expr -> Result<i64, Box<dyn Error>>:
	Expr '+' Term {
		Ok($1? + $3?)
	}
	| Term {
		$1
	}
	;

Term -> Result<i64, Box<dyn Error>>:
	'INT' {
        parse_int($lexer.span_str($1.map_err(|_| "<evaluation aborted>")?.span()))
	}
	| '(' Expr ')' {
		$2
	}
	;
Bool -> Result<bool, Box<dyn Error>>:
	'true' {
		Ok(true)
	}
	| 'false' {
		Ok(false)
	}
	;
%%
// Any imports here are in scope for all the grammar actions above.

use std::error::Error;
use cfgrammar::Span;

fn flatten<T>(lhs: Result<Vec<T>, Box<dyn Error>>, rhs: Result<T, Box<dyn Error>>)
           -> Result<Vec<T>, Box<dyn Error>>
{
    let mut flt = lhs?;
    flt.push(rhs?);
    Ok(flt)
}

fn parse_int(s: &str) -> Result<i64, Box<dyn Error>> {
    match s.parse::<i64>() {
        Ok(val) => Ok(val),
        Err(_) => {
            Err(Box::from(format!("{} cannot be represented as a i64", s)))
        }
    }
}

#[derive(Debug)]
pub enum Toplevel {
	States{name: Span, elements: Vec<StateElement>},
	Spawn{name: Span, elements: Vec<SpawnElement> },
	Enum{name: Span, elements: Vec<Span>},
	Function { name: Span, body: Vec<Word> },
}

#[derive(Debug)]
pub enum FunctionRef {
	Name(Span),
	Inline(Vec<Word>)
}

#[derive(Debug)]
pub enum StateElement {
	State { sprite: (Span,Span), directional: bool, timeout: i64, think: FunctionRef, action: FunctionRef, next: Span},
	Label(Span)
}

#[derive(Debug)]
pub struct SpawnElement {
	pub directional: bool,
	pub id: i64,
	pub state: Span,
	pub drop: Span,
}

#[derive(Debug, Clone)]
pub enum TypedInt {
	U8(u8),
	I32(i32),
}

#[derive(Debug, Clone)]
pub enum TypeName {
	U8,
	I32,
}

#[derive(Debug,Clone)]
pub enum Word {
	Push(TypedInt),
	PushStateLabel(Span),
	PushEnum(Span,Span), // FIXME: only u8 for now
	Trap,
	Not,
	If(Vec<Word>),
	GoState,
	Stop,
	Add,
	Call,
	WordList(Vec<Word>),
}