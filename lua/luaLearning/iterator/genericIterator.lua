---
--- Generated by Luanalysis
--- Created by Admin.
--- DateTime: 2021/8/13 11:51
---
-- 无状态的迭代器
-- 无状态的迭代器是指不保留任何状态的迭代器，因此在循环中我们可以利用无状态迭代器避免创建闭包花费额外的代价。

function square(iteratorMaxCount, currentNumber)
    if currentNumber < iteratorMaxCount then
        currentNumber = currentNumber + 1
        return currentNumber, currentNumber*currentNumber
    end
end

for i, n in square, 3, 0 do
    print(i, n)
end



--[[
多状态的迭代器
很多情况下，迭代器需要保存多个状态信息而不是简单的状态常量和控制变量，
最简单的方法是使用闭包，
还有一种方法就是将所有的状态信息封装到table内，]]


array = {"Lua", "Tutorial"}

function elementIterator(collection)
    local index = 0
    local count = #collection
    -- 闭包函数
    return function()
        index = index + 1
        if index <= count then
            --  返回迭代器的当前元素
            return collection[index]
        end
    end
end

for element in elementIterator(array)
do
    print(element)
end