import Parser from 'tree-sitter';
import Jack from 'tree-sitter-jack';
import { readFile } from 'fs/promises';
import { argv, exit } from 'process';

const parser = new Parser();
parser.setLanguage(Jack);

if (!argv[2]) {
  exit(65);
}

const contents = await readFile(argv[2], 'utf-8');
const tree = parser.parse(contents);
console.log(tree.rootNode.toString());
