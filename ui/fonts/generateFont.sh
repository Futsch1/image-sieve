#!/bin/bash
rm *.ttf
python3 generateSymbols.py
wget https://github.com/googlefonts/noto-emoji/raw/main/fonts/NotoColorEmoji.ttf
pyftsubset NotoColorEmoji.ttf --unicodes-file=symbols.txt --output-file=NotoColorEmoji.subset.ttf
