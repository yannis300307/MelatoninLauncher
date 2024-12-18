class_name VdfParser

enum ParsingError {
	SUCCESS = 0,
	UNMATCHING_QUOTES = 1,
	UNRECONISED_TOKEN = 2,
	UNMATCHING_BRACKETS = 3,
}

# A recursive basic and unsafe parser of Valve Data Format (VDF) made from my understanding of it
func parse_vdf_expression(vdf: String):
	var result = get_tokens(vdf)
	
	match result[0]:
		ParsingError.SUCCESS:
			pass
		ParsingError.UNMATCHING_QUOTES:
			return ParsingError.UNMATCHING_QUOTES
		ParsingError.UNRECONISED_TOKEN:
			return ParsingError.UNRECONISED_TOKEN
	
	return parse_single_expression(result[1])
	
func parse_single_expression(expr: Array):
	var dict = {}
	var i = 0
	while i < len(expr):
		var key = expr[i].substr(1,len(expr[i])-2)
		
		if expr[i+1].begins_with('"'):
			dict[key] = expr[i+1].substr(1,len(expr[i+1])-2)
			i+= 2
		else:
			if expr[i+1] == "{":
				var brackets_counter = 1
				i += 2
				var beginning = i
				while brackets_counter != 0 and i < len(expr):
					if expr[i] == "{":
						brackets_counter += 1
					elif expr[i] == "}":
						brackets_counter -= 1 
					i += 1
				if brackets_counter:
					return ParsingError.UNMATCHING_BRACKETS
				
				var parsed_value = parse_single_expression(expr.slice(beginning, i-1))
				if parsed_value is int and parsed_value == ParsingError.UNMATCHING_BRACKETS:
					return ParsingError.UNMATCHING_BRACKETS
				
				dict[key] = parsed_value
	return dict

func get_tokens(expression: String):
	var tokens = []
	
	var i = 0
	while i < len(expression):
		match expression[i]:
			"{":
				tokens.append("{")
			"}":
				tokens.append("}")
			'"':
				tokens.append('"')
				i+=1
				while i < len(expression) and expression[i] != '"' and expression != "\n":
					tokens[-1] += expression[i]
					i += 1
					
				if expression[i-1] == "\n" or i-1 >= len(expression):
					return [ParsingError.UNMATCHING_QUOTES, i]
				tokens[-1] += expression[i]

			"\n":
				pass
			"\t":
				pass
			" ":
				pass
			_:
				return [ParsingError.UNRECONISED_TOKEN, i, expression[i]]
			
		i += 1
	return [ParsingError.SUCCESS, tokens]
