const test = require("../index.js")

console.log(test.sum(1, 2))

// export function readFileAsync(path: string): Promise<Buffer>

async function asyncRead() {
    var buff = await test.readFileAsync("E:\\syncher.nomad")
    console.log(String(buff))
}

asyncRead()