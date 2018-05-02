class CompileTimeError(Exception):
	def __init__(self, error_type, problematic_token, error_message):
		total_message = error_type + ' at ??:??\n'
		total_message += 'at ' + str(problematic_token) + '\n'
		total_message += error_message
		super().__init__(total_message)
		
class SyntaxError(CompileTimeError):
	def __init__(self, state_chain, problematic_token, expected=None):
		if(expected == None):
			expected = problematic_token
			problematic_token = state_chain
			state_chain = []
		super().__init__('SYNTAX ERROR', problematic_token,
			'got ' + problematic_token.label + ', expected ' + expected + '.\n' +
			'State chain: ' + str(state_chain))


