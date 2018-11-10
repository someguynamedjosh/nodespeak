class CompileTimeError(Exception):
	def __init__(self, error_type, problematic_token, error_message):
		total_message = error_type + ' at ??:??\n'
		total_message += 'at ' + str(problematic_token) + '\n'
		total_message += error_message
		super().__init__(total_message)
		
class SyntaxError(CompileTimeError):
	def __init__(self, problematic_token, expected):
		super().__init__('SYNTAX ERROR', problematic_token,
			'got ' + str(problematic_token) + ', expected ' + expected + '.')