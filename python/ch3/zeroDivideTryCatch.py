def spam(divideBy):
    try:
        return 42 / divideBy
    except ZeroDivisionError:
        print('Error : Invalid divide')

print(spam(2))
print(spam(12))
print(spam(0)) # Error : Invalid divide
                # None 在运行那些代码之后，执行照常继续。
 
print(spam(1))
