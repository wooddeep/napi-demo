const cluster = require("cluster")
const os = require("os")

const Koa = require('koa')
const app = new Koa()

const test = require("../index.js")


// export function readFileAsync(path: string): Promise<Buffer>

async function asyncRead() {
    var buff = await test.readFileAsync("E:\\syncher.nomad")
    console.log(String(buff))
}

//asyncRead()


if (cluster.isMaster) { // main process
    console.log(`## parent process id: ${test.show()}\n`)

    for (var i = 0, n = 2 /*os.cpus().length*/; i < n; i += 1) {
        cluster.fork(); // start child process
    }

    cluster.on("exit", (worker, code, signal) => { // start again when one child exit!
        cluster.fork();
    })

} else {

    test.start()

    console.log(`## child process id: ${test.show()}`)

    app.listen(5050, async () => {
        console.log("# master start ok!")
    })

}