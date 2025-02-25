// Reflect 对象

const obj = {
    foo: '123',
    bar: '456'
}

let proxy = new Proxy(obj, {
    get(target, p) {
        console.log('watch logic~')
        return Reflect.get(target, p)
    }
});

console.log(proxy.foo)

const girl = {
    name: 'vanida',
    age: 19
}


console.log('name' in girl)
console.log(delete girl['age'])
console.log(Object.keys(girl))