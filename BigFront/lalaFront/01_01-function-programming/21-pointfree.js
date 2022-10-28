// 把一个字符串中的首字母提取并转换成大写, 使用. 作为分隔符
// world wild web ==> W. W. W

const fp = require('lodash/fp')

// 两个map 循环
const firstLetterToUpper = fp.flowRight(fp.join('. '), fp.map(fp.first), fp.map(fp.toUpper), fp.split(' '))

// 一个map 循环
const secondLetterToUpper = fp.flowRight(fp.join('. '), fp.map(fp.flowRight(fp.first, fp.toUpper)), fp.split(" "))

console.log(firstLetterToUpper('world wild web'))
console.log(secondLetterToUpper('world wild web'))
