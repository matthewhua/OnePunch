// 箭头函数与 this
// 箭头函数不会改变 this 指向

const person = {
    name: 'tom',
    sayHi: function () {
        console.log(`hi, my name is ${this.name}`)
    },

    sayH2: () => { //指向函数里
        console.log(`h2, my name is ${this.name}`)
    },

    sayHiAsync: function () {
        setTimeout(() => {
            console.log(this.name)
        }, 1000)
    }
}

person.sayHi()
person.sayH2()
person.sayHiAsync()