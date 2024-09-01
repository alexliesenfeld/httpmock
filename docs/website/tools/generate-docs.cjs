const fs = require('fs');
const path = require('path');

const Handlebars = require('handlebars');
Handlebars.registerHelper('eq', (a, b) => a === b);

function readJsonFile(filename) {
    const rawData = fs.readFileSync(filename);
    return JSON.parse(rawData);
}

function deepenMarkdownHeaders(markdownText) {
    return markdownText.split('\n').map(line => {
        // Check if the line starts with one or more '#'
        if (line.startsWith('#')) {
            return '###' + line;  // Add two more '#' to deepen the header
        }
        return line;
    }).join('\n');
}

function generateMarkdownDocs() {
    const methodDocs = readJsonFile(process.argv[2]);
    Object.keys(methodDocs.then).forEach(key => {
        methodDocs.then[key] = deepenMarkdownHeaders(methodDocs.then[key]);
    });
    Object.keys(methodDocs.when).forEach(key => {
        methodDocs.when[key] = deepenMarkdownHeaders(methodDocs.when[key]);
    });

    const templatesDir = process.argv[3];

    fs.readdir(templatesDir, (err, files) => {
        if (err) return console.error(err);

        files.forEach(file => {
            const filePath = path.join(templatesDir, file);
            fs.readFile(filePath, 'utf8', (err, content) => {
                if (err) return console.error(err);

                const template = Handlebars.compile(content);
                const result = template({ docs: methodDocs });

                const fileName = `${process.argv[4]}/${file}`;
                if (fs.existsSync(fileName)) {
                    fs.unlinkSync(fileName);
                }

                console.log("writing: " + fileName)
                fs.writeFileSync(fileName, result);
            });
        });
    });
}

generateMarkdownDocs();