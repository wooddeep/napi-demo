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
}).get('write', async (ctx) => {
    backend.sendData(Buffer.from("hello world!"))
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

if (cluster.isMaster) { // main process
    //backend.init()

    backend.masterInit()

    //backend.testShmWriteThread()

    // test.callThreadsafeFunction(async () => {
    //     console.log(`## called by rust...... time: ${new Date()}`)
    // })

    for (var i = 0, n = child_proc_num ; i < n; i += 1) {
        var new_worker_env = {};
        new_worker_env["WORKER_INDEX"] = i;
        cluster.fork(new_worker_env); // start child process
    }

    cluster.on("exit", (worker, code, signal) => { // start again when one child exit!
        cluster.fork();
    })

} else {

    //backend.testShmReadThread()
    backend.workerInit()

    console.log("WORKER_INDEX", process.env["WORKER_INDEX"])
    app.listen(5050, async () => {
        //console.log("# child start ok!")
    })

}

