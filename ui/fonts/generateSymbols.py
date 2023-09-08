with open('../help.slint') as f:
    content = f.read()
    symbols = set()
    for char in content:
        if ord(char) > 127:
            symbols.add(ord(char))

with open('symbols.txt', 'w') as f:
    f.write('\n'.join([f'U+{symbol:X}' for symbol in sorted(list(symbols))]))
    f.close()
