// 函数组合要满足结合律

const _ = require('lodash')

const f = _.flowRight(_.toUpper, _.flowRight(_.first, _.reverse))
const f2 = _.flowRight(_.flowRight(_.toUpper, _.first), _.reverse)

console.log(f(['one', 'two', 'three']))
console.log(f2(['one', 'two', 'three']))