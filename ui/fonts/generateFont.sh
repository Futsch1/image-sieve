#!/bin/bash
wget https://github.com/googlefonts/noto-emoji/raw/main/fonts/NotoColorEmoji.ttf
wget https://github.com/notofonts/notofonts.github.io/raw/main/fonts/NotoSans/full/ttf/NotoSans-Regular.ttf
pyftsubset NotoColorEmoji.ttf --unicodes-file=symbols.txt --output-file=NotoColorEmoji.subset.ttf
#ttx -o NotoImageSieve.ttf -m NotoColorEmoji.subset.ttf NotoSans-Regular.ttx
#ttx NotoColorEmoji.subset.ttf
ttx NotoSans-Regular.otf
ttx -o NotoSans-Regular.patched.ttf NotoSans-Regular.ttx
#ttx -o NotoImageSieve.ttf -m NotoSans-Regular.ttf NotoColorEmoji.subset.ttx 
pyftmerge NotoSans-Regular.patched.ttf NotoColorEmoji.subset.ttf --output-file=NotoImageSieve.ttf
