
function ajax (url) {
    return new Promise(function (resolve, reject) {
        var xhr = new XMLHttpRequest()
        xhr.open('GET', url)
        xhr.responseType = 'json'
        xhr.onload = function () {
            if (this.status === 200) {
                resolve(this.response)
            } else {
                reject(new Error(this.statusText))
            }
        }
        xhr.send()
    })
}

ajax('/api/users.json')
    .then(function (value) {
        console.log(1111)
        return ajax('/api/urls.json')
    })// => Promise
    .then(function (value) {
        console.log(2222)
        console.log(value)
        return ajax('/api/urls.json')
    }) // => Promise
    .then(function (value) {
        console.log(3333)
        return ajax('/api/urls.json')
    }) // => Promise
    .then(function (value) {
        console.log(4444)
        return 'foo'
    }) // => Promise
    .then(function (value) {
        console.log(5555)
        console.log(value)
    })
