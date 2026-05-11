import re
import json
import sys

def pb_to_json_obj(text):
    # Better tokenization
    token_patterns = [
        ('LBRACE', r'\{'),
        ('RBRACE', r'\}'),
        ('COLON', r':'),
        ('LBRACKET', r'\['),
        ('RBRACKET', r'\]'),
        ('STRING', r'"(?:\\.|[^"])*"'),
        ('NUMBER', r'[-]?[0-9][0-9a-zA-Z.]*'),
        ('ID', r'[a-zA-Z_][a-zA-Z0-9_.]*'),
        ('BOOL', r'\b(true|false)\b'),
    ]
    
    combined_pattern = '|'.join(f'(?P<{name}>{pat})' for name, pat in token_patterns)
    
    tokens = []
    for m in re.finditer(combined_pattern, text):
        kind = m.lastgroup
        value = m.group(kind)
        tokens.append((kind, value))

    def parse_msg(it):
        obj = {}
        while True:
            try:
                kind, token = next(it)
            except StopIteration:
                break
                
            if kind == 'RBRACE':
                break
            
            key = token
            if kind == 'LBRACKET':
                # Extension
                k_kind, key = next(it)
                # skip RBRACKET
                try:
                    nk, nv = next(it)
                    while nv != ']':
                        key += nv
                        nk, nv = next(it)
                except StopIteration:
                    pass
            
            try:
                skind, sep = next(it)
            except StopIteration:
                obj[key] = True
                break

            if skind == 'COLON':
                vkind, value_token = next(it)
                if vkind == 'LBRACE':
                    value = parse_msg(it)
                else:
                    if vkind == 'STRING':
                        value = value_token[1:-1].replace('\\"', '"')
                    elif vkind == 'BOOL':
                        value = (value_token == 'true')
                    else:
                        try:
                            if '.' in value_token:
                                value = float(value_token)
                            else:
                                if value_token.startswith('0x'):
                                    value = int(value_token, 16)
                                else:
                                    value = int(value_token)
                        except ValueError:
                            value = value_token
                
                if key in obj:
                    if not isinstance(obj[key], list):
                        obj[key] = [obj[key]]
                    obj[key].append(value)
                else:
                    obj[key] = value
            elif skind == 'LBRACE':
                value = parse_msg(it)
                if key in obj:
                    if not isinstance(obj[key], list):
                        obj[key] = [obj[key]]
                    obj[key].append(value)
                else:
                    obj[key] = value
            else:
                # Key without value
                obj[key] = True
                # Put back the token if it's likely a new key
                # (Not easy with this iterator, but let's assume it's okay)
        return obj

    it = iter(tokens)
    try:
        kind, first = next(it)
        if kind == 'LBRACE':
            return parse_msg(it)
        else:
            # Sequence of fields
            return parse_msg(iter([(kind, first)] + list(it)))
    except StopIteration:
        return {}

def process_file(input_path, output_path):
    with open(input_path, 'r') as f:
        lines = f.readlines()
    
    all_logs = []
    for line in lines:
        line = line.strip()
        if not line: continue
        
        start_idx = line.find('{')
        if start_idx != -1:
            prefix = line[:start_idx]
            data_part = line[start_idx:]
            
            meta = {}
            ts_match = re.search(r'(\d{2}:\d{2}:\d{2}\.\d{3})', prefix)
            if ts_match: meta['timestamp'] = ts_match.group(1)
            role_match = re.search(r'roleId:(\d+)', prefix)
            if role_match: meta['roleId'] = role_match.group(1)

            json_data = pb_to_json_obj(data_part)
            all_logs.append({
                "meta": meta,
                "data": json_data
            })
        else:
            all_logs.append({"raw": line})
            
    with open(output_path, 'w') as f:
        json.dump(all_logs, f, indent=2, ensure_ascii=False)

if __name__ == "__main__":
    process_file(sys.argv[1], sys.argv[2])
