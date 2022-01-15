def spam(divideBy):
    return 42 / divideBy

print(spam(2))
print(spam(12))
print(spam(0)) # Traceback (most recent call last): 后面不执行了
print(spam(1))

# 错误可以由 try 和 except 语句来处理。那些可能出错的语句被放在 try 子句中。
# 如果错误发生，程序执行就转到接下来的 except 子句开始处。