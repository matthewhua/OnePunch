def spam():
    print(eggs) #Error!
    eggs = 'spam local'

eggs = 'global'
spam()

# 因此认为 eggs 变量是局部变量。但是因为 print(eggs)的执行在 eggs 赋值之前，局部变
# 量 eggs 并不存在。Python 不会退回到使用全局 eggs 变量