from token import *
from errors import *
from scope import *

def process(token):
    if(token.role == TokenRole.NAMESPACE):
        scope = Scope()
        for sub in token.subs:
            if(sub.__name__() == 'statement_typedef'):
                