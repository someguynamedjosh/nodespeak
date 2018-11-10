from enum import Enum, auto

# Things higher up in the list have higher priority. The priorities go from 255 -> 0 (or whichever is the last element in the enum.)
class TokenType(Enum):
    def _generate_next_value_(name, start, count, last_values): # Give the highest numbers for types that are higher on the list.
        return 255 - count
    
    KEYWORD_IF = auto()
    KEYWORD_ELIF = auto()
    KEYWORD_ELSE = auto()
    KEYWORD_FOR = auto()
    KEYWORD_WHILE = auto()
    KEYWORD_DO = auto()
    KEYWORD_TYPEDEF = auto()
    KEYWORD_BREAK = auto()
    KEYWORD_RETURN = auto()
    KEYWORD_DEF = auto()
    
    OCTAL_NUMBER = auto()
    BINARY_NUMBER = auto()
    HEX_NUMBER = auto()
    NUMBER = auto()
    
    DOT_OPERATOR = auto()
    COLON = auto()
    SEMICOLON = auto()
    COMMA = auto()
    OPEN_PAREN = auto()
    CLOSE_PAREN = auto()
    OPEN_BRACE = auto()
    CLOSE_BRACE = auto()
    OPEN_BRACKET = auto()
    CLOSE_BRACKET = auto()
    
    MULTIPLY_OPERATOR = auto()
    ADD_OPERATOR = auto()
    MINUS_OPERATOR = auto()
    
    BITWISE_NOT_OPERATOR = auto()
    BITWISE_OPERATOR = auto()
    NOT_OPERATOR = auto()
    BOOLEAN_OPERATOR = auto()
    COMPARISON_OPERATOR = auto()
    
    ASSIGNMENT_OPERATOR = auto()
    ASSIGN_WITH_MATH_OPERATOR = auto()
    
    IDENTIFIER = auto()
    
    IGNORE = auto()
    EOF = auto()