//
// Created by Admin on 2021/11/15.
//
#include <iostream>
#include <stack>
#include <string>
using namespace std;

// 栈容器，先进后出
//存储基础数据类型
void test01()
{
    stack<int> s;
    s.push(10);
    s.push(20);
    s.push(30);
    s.push(40);
    s.push(50);

    //输出栈中元素
    while (!s.empty())
    {
        //输出栈顶元素
        cout << s.top() << " ";
        //弹出栈顶元素
        s.pop();
    }
    cout << "size: " << s.size() << endl;
}

class Maker
{
public:
    Maker(const string &name, int age) : name(name), age(age) {}

public:
    string name;
    int age;
};

// 存储对象
void test02()
{
    stack<Maker> s;
    s.push(Maker("aaa", 18));
    s.push(Maker("bbb", 19));
    s.push(Maker("ccc", 20));
    s.push(Maker("ddd", 90));

    while (!s.empty())
    {
        cout << "Name:" << s.top().name << " Age:" << s.top().age << endl;
        s.pop();
    }

}


int main()
{
    test02();
    return EXIT_SUCCESS;
}
