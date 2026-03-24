const fs = require("fs");
const path = require("path");
const qs = require("qs");

const fixturePath = path.join(__dirname, "..", "test_cases.json");
const testCases = JSON.parse(fs.readFileSync(fixturePath, "utf8"));

const results = testCases.map((testCase) => ({
    encoded: qs.stringify(testCase.data),
    decoded: qs.parse(testCase.encoded),
}));

process.stdout.write(JSON.stringify(results));
