const cluster = require("cluster")
const os = require("os")

const Koa = require('koa')
const app = new Koa()

const test = require("../index.js")
const {initProcInfo, runServer, testShmWrite, testShmRead, printThreadId, testShmWriteThread} = require("../index");


// export function readFileAsync(path: string): Promise<Buffer>

async function asyncRead() {
    var buff = await test.readFileAsync("E:\\syncher.nomad")
    console.log(String(buff))
}

//asyncRead()

const Router = require('koa-router')

let home = new Router()

// 子路由1
home.get('/', async (ctx) => {
    let html =
        `
    <ul>
      <li><a href="/page/helloworld">/page/helloworld</a></li>
      <li><a href="/page/404">/page/404</a></li>
    </ul>
  `
    ctx.body = html
})

// 子路由2
let page = new Router()
page.get('/404', async (ctx) => {
    ctx.body = '404 page!'
}).get('/helloworld', async (ctx) => {
    ctx.body = 'helloworld page!'
})

// 装载所有子路由
let router = new Router()
router.use('/', home.routes(), home.allowedMethods())
router.use('/page', page.routes(), page.allowedMethods())

// 加载路由中间件
app.use(router.routes()).use(router.allowedMethods())

const child_proc_num = 1 // /*os.cpus().length*/


if (cluster.isMaster) { // main process
    console.log(`## parent process id: ${test.show()}\n`)

    test.testShmWriteThread()

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

    test.testShmReadThread()
    console.log("WORKER_INDEX", process.env["WORKER_INDEX"])
    app.listen(5050, async () => {
        //console.log("# child start ok!")
    })

}

