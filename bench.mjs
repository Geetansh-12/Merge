import { marked } from "marked";
let input = "";
process.stdin.on("data", d => input += d);
process.stdin.on("end", () => process.stdout.write(marked.parse(input)));
