def eggs(someParameter):
    someParameter.append('hello')

# 尽管 spam 和 someParameter 包含了不同的引用，但它们都指向相同的列表。这就是
# 为什么函数内的 append('Hello')方法调用在函数调用返回后，仍然会对该列表产生影响。
spam = [1, 2, 3]
eggs(spam)
print(spam)