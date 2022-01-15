import copy

spam = ['A', 'B', 'C','D', 'E', 'F']
cheese = copy.copy(spam) #浅拷贝
deep = copy.deepcopy(spam) #如果要复制的列表中包含了列表，那就使用 copy.deepcopy()函数来代替。

spam.remove('C')

print(spam)
print(cheese)
print(deep)
