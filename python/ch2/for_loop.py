#  for 关键字；
#  一个变量名；
#  in 关键字；
#  调用 range()方法，最多传入 3 个参数；
#  冒号；
#  从下一行开始，缩退的代码块（称为 for 子句）。

print('My name is')
for i in range(5):
    print(('Jimmy Five Times (' + str(i) + ')'))



total = 0
for num in range(101):
    total = total + num
print(total)

# range()的开始、停止和步长参数 
for i in range(12, 16): #12 到16, 步长为1
    print(i)

for i in range(0, 10, 2): #0到10, 步长为2
    print(i)