spam = {'name' : 'Pooka', 'age' : 5}
if 'color' not in spam:
    spam['color'] = 'red'


# setdefault()方法提供了一种方式，在一行中完成这件事。传递给该方法的第一
# 个参数，是要检查的键。第二个参数，是如果该键不存在时要设置的值。如

spam.setdefault('food', 'meat')

print(spam)
