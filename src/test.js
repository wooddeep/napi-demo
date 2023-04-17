const cluster = require("cluster")
const Router = require('koa-router')
const Koa = require('koa')
const app = new Koa()
const backend = require("../index.js")
const {initProcInfo, runServer, testShmWrite, testShmRead, printThreadId, testShmWriteThread, callback} = require("../index");
const {Buffer} = require("memfs/lib/internal/buffer");

let page = new Router()
page.get('404', async (ctx) => {
    ctx.body = '404 page!'
}).get('hello', async (ctx) => {
    ctx.body = 'hello world page!'
}).get('require', async (ctx) => {
    await backend.testSemaRequire()
    ctx.body = 'testSemaRequire response'
}).get('release', async (ctx) => {
    await backend.testSemaRelease()
    ctx.body = 'testSemaRelease response'
}).get('read', async (ctx) => {
    await backend.testShmRead()
    ctx.body = 'testShmRead response'
}).get('write', async (ctx) => {
    await backend.testShmWrite()
    ctx.body = 'testShmWrite response'
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

    let child_map = new Map()
    for (var i = 0, n = child_proc_num ; i < n; i += 1) {
        let new_worker_env = {};
        new_worker_env["WORKER_INDEX"] = i;
        let child = cluster.fork(new_worker_env); // start child process
        child_map.set(child.process.pid, i)
    }

    cluster.on("exit", (worker, code, signal) => { // start again when one child exit!
        let new_worker_env = {};
        let index = child_map.get(worker.process.pid)
        new_worker_env["WORKER_INDEX"] = index;
        child_map.delete(worker.process.pid)
        let child = cluster.fork(new_worker_env);
        child_map.set(child.process.pid, index)
    })

} else {
    backend.workerInit()

    subscribe(async () => {
        let data = await backend.testShmRead();
        console.log(`## process id: ${process.pid}; data = ${data}`)
    });

    process.WORKER_INDEX = process.env["WORKER_INDEX"]
    console.log("WORKER_INDEX", process.env["WORKER_INDEX"])
    app.listen(5050, async () => {
        console.log("# child start ok!")
    })

}

