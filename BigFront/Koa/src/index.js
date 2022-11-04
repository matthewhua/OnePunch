import jsonutil from 'koa-json'
import compose from 'koa-compose'
import koaBody from "koa-body";
import * as path from "path";
import cors from '@koa/cors'
import Koa from 'koa'
import router from 'koa-router'
import statics from 'koa-static'
import helmet from 'koa-helmet'

let app = new Koa();

/**
 * 使用koa-compose 集成中间件
 */
const  middleware = compose([
    koaBody(),
    statics(path.join(__dirname, '../public'))
    cors(),
    jsonutil({ pretty : false, param: 'pretty'})
    helmet()
])

app.use(middleware)
    .use(router())

app.listen(3000)

/*
const Koa = require('koa')
const Router = require('koa-router')
const Cors = require('@koa/cors')
const KoaBody = require('koa-body')
const json = require('koa-json')

const app = new Koa()
const router = new Router()

router.prefix('/api')

router.get('/', ctx => {
    console.log(ctx.request)
    ctx.body = 'Hello World'
})

router.get('/api', ctx => {
    // get params
    const params = ctx.request.query
    console.log(params)
    //name: 'imooc', age:'28'
    console.log(params.name, params.age)
    console.log(ctx.request)
    ctx.body = 'Hello AP!!!!!'
})

router.post('/post', async (ctx) => {
    let { body } = ctx.request;
    console.log(body)
    console.log(ctx.request)
    ctx.body = {
        ...body //扩展运算符
    }
})

app.use(KoaBody())
    .use(Cors())
    .use(json({ pretty: false, param: 'pretty'}))
    .use(router.routes())
    .use(router.allowedMethods())


app.listen(3000)*/
