// lodash 和 lodash/fp 模块中 map 方法的区别

/*const _ = require('lodash')

console.log(_.map(['23', '8', '11'], parseInt))*/
// // parseInt('23', 0, array) 从 0开始 23
// // parseInt('8', 1, array)  没有1进制 NaN
// // parseInt('11', 2, array) 二进制 3


const fp = require('lodash/fp')

console.log(fp.map(parseInt, ['23', '8', '10']))
