const cluster = require("cluster")
const Router = require('koa-router')
const Koa = require('koa')
const app = new Koa()
const backend = require("../index.js")
const {initProcInfo, runServer, testShmWrite, testShmRead, printThreadId, testShmWriteThread} = require("../index");
const {Buffer} = require("memfs/lib/internal/buffer");

let page = new Router()
page.get('404', async (ctx) => {
    ctx.body = '404 page!'
}).get('hello', async (ctx) => {
    ctx.body = 'hello world page!'
}).get('test_req', async (ctx) => {
    await backend.testSemaRequire()
    ctx.body = 'testSemaRequire response'
}).get('test_rel', async (ctx) => {
    await backend.testSemaRelease()
    ctx.body = 'testSemaRelease response'
}).get('write', async (ctx) => {
    let buff = Buffer.from("hello world!")
    let index = Number.parseInt(process.env["WORKER_INDEX"])
    backend.sendData(index, buff, buff.length)
    ctx.body = 'write page!'
})

let router = new Router()
router.use('/', page.routes(), page.allowedMethods())

// 加载路由中间件
app.use(router.routes()).use(router.allowedMethods())

const child_proc_num = 1 // /*os.cpus().length*/

process.on("SIGINT", () => {
    backend.processExit()
    process.exit()
});

process.on("beforeExit", (code) => {
    console.log("## pre exit in node...")
    backend.processExit()
})

function subscribe(callback) {
    backend.callNodeFunc().then((data) => {
        callback(data)
        subscribe(callback)
    })
}

if (cluster.isMaster) { // main process
    backend.masterInit()

    subscribe((data) => {
        console.log(`## data = ${data}`)
    });

    for (var i = 0, n = child_proc_num ; i < n; i += 1) {
        var new_worker_env = {};
        new_worker_env["WORKER_INDEX"] = i;
        cluster.fork(new_worker_env); // start child process
    }

    cluster.on("exit", (worker, code, signal) => { // start again when one child exit!
        cluster.fork();
    })

} else {

    backend.workerInit()
    console.log("WORKER_INDEX", process.env["WORKER_INDEX"])
    app.listen(5050, async () => {
        //console.log("# child start ok!")
    })

}

