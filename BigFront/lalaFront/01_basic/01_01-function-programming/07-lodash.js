const _ = require('lodash')

const array = ['java', 'Matthew', 'lucy', 'kate']

console.log(_.first(array))
console.log(_.last(array))

console.log(_.toUpper(_.first(array)))

console.log(_.reverse(array))

const r = _.each(array, (item, index) => {
    console.log(item, index)
})

console.log(r)


let contains = _.find(array, 'Matthew');
console.log(contains)