import typing
import textwrap
from itertools import chain


def get_chapter(heading: str, level: int, data: typing.List[str]) -> typing.List[str]:
    """
    Get the chapter from the data.
    """
    start = data.index('#' * level + ' ' + heading + '\n')
    for end, line in enumerate(data[start + 1:]):
        if line.startswith('#' * (level)) or line.startswith('#' * (level - 1)):
            return data[start + 1:start+end]
    return data[start + 1:]


def process_line(s: str) -> typing.List[str]:
    """
    Process a line.
    """
    s = s.replace('"', '\\"')
    if len(s) == 1:
        return [""]
    else:
        return textwrap.wrap(s, width=180)


with open("README.md", "r") as f:
    lines = f.readlines()

chapters = ["📷 📹 Images", '📅 Events', '💾 Sieve', '⚙ Settings']
chapter_help = {}
for chapter in chapters:
    chapter_lines = get_chapter(chapter, 3, lines)
    chapter_content = list(chain(*[process_line(line) for line in filter(
        lambda x: not x.startswith('!['), chapter_lines)]))
    chapter_help[chapter] = '\\n'.join(chapter_content)

with open('ui/help.60', 'r') as f:
    lines = f.readlines()

for chapter in chapter_help:
    for number, line in enumerate(lines):
        if chapter in line:
            print(f'Found {chapter} at line {number}')
            for text_number, text_line in enumerate(lines[number + 1:]):
                if 'text:' in text_line:
                    print(f'Inserting text at line {number + text_number + 1}')
                    lines[number +
                          text_number + 1] = f'            Text {{ text: "{chapter_help[chapter]}";\n'
                    break
            break

with open('ui/help.60', 'w') as f:
    f.writelines(lines)
