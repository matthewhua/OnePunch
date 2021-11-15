//
// Created by $Matthew on 2021-11-07.
//
#define _CRT_SECURE_NO_WARNINGS
#include<string>
#include <vector> // 容器
#include <algorithm> //算法
#include<iostream>
using namespace std;


//加入算法的回调函数
void MyPrint(int val)
{
    cout << val << " ";
}
// 1. 存储基础数据类型
void test01()
{
    // 容器
    vector<int> v;
    v.push_back(10);
    v.push_back(20);
    v.push_back(30);
    v.push_back(40);
    v.push_back(50);

    //获取开始位置的迭代器
    vector<int>::iterator  begin = v.begin();
    //获取结束位置的迭代器
    vector<int>::iterator end= v.end();
    //遍历算法
    for_each(begin, end, MyPrint);
    cout << endl;
}

// 2. 容器存储对象
class Maker
{
public:
    Maker( string name, int age) {
        this->name = name;
        this->age = age;
    }

public:
    string name;
    int age;
};
ostream &operator<<(ostream &out, Maker &m)
{
    out << "Name:" << m.name << "Age:" << m.age <<endl;
    return out;
}

void  test02()
{
    vector<Maker> v;
    //忘容器中存储对象
    v.push_back(Maker("悟空",  18));
    v.push_back(Maker("小林", 19));

    v.push_back(Maker("短笛", 180));

    //获取开始和结束位置的迭代器
    vector<Maker>::iterator begin = v.begin();
    vector<Maker>::iterator end = v.end();
    while (begin != end)
    {
        cout << (*begin);
        begin++;
    }
}

//3. 存储对象的指针
void test03()
{
    vector<Maker*> v;
    //创建数据
    Maker *m1 = new Maker("悟空", 18);
    Maker *m2 = new Maker("小林", 19);
    Maker *m3 = new Maker("贝吉塔",200 );
    Maker *m4 = new Maker("龟仙人",180 );
    Maker *m5 = new Maker("短笛", 18);

    v.push_back(m1);
    v.push_back(m2);
    v.push_back(m3);
    v.push_back(m4);
    v.push_back(m5);

    vector<Maker*>::iterator begin = v.begin();
    vector<Maker*>::iterator end = v.end();
    while (begin != end)
    {
        cout << (*begin)->name << " " << (*begin)-> age << endl;
        ++begin;
    }

    delete m1;
    delete m2;
    delete m3;
    delete m4;
    delete m5;
}

// 4. 容器嵌套容器
void test04()
{
    vector<vector<int>> vs;
    vector<int> v1;
    vector<int> v2;
    vector<int> v3;
    vector<int> v4;
    vector<int> v5;

    for (int i = 0; i < 5; ++i) {
        v1.push_back(i + 10);
        v2.push_back(i + 10);
        v3.push_back(i + 10);
        v4.push_back(i + 10);
        v5.push_back(i + 10);
    }

    vs.push_back(v1);
    vs.push_back(v2);
    vs.push_back(v3);
    vs.push_back(v4);
    vs.push_back(v5);
    vector<vector<int>>::iterator begin = vs.begin();
    vector<vector<int>>::iterator end = vs.end();

    while (begin != end)
    {
        vector<int>::iterator sbegin = (*begin).begin();
        vector<int>::iterator send = (*begin).end();

        while (sbegin != send)
        {
            cout << *sbegin << " ";
            ++sbegin;
        }
        cout << endl;
        ++begin;
    }
}

/*
查找和替换
int find(const string& str, int pos = 0) const; //查找str第一次出现位置,从pos开始查找
int find(constchar* s, int pos = 0) const;  //查找s第一次出现位置,从pos开始查找
int find(constchar* s, int pos, int n) const;  //从pos位置查找s的前n个字符第一次位置
int find(constchar c, int pos = 0) const;  //查找字符c第一次出现位置
int rfind(conststring& str, int pos = npos) const;//查找str最后一次位置,从pos开始查找
int rfind(constchar* s, int pos = npos) const;//查找s最后一次出现位置,从pos开始查找
int rfind(constchar* s, int pos, int n) const;//从pos查找s的前n个字符最后一次位置
int rfind(constchar c, int pos = 0) const; //查找字符c最后一次出现位置
string& replace(int pos, int n, const string& str); //替换从pos开始n个字符为字符串str
string& replace(int pos, int n, const char* s); //替换从pos开始的n个字符为字符串s

*//*

void test05()
{
    string s = "abcdefgd";
    cout << s.find('d') << endl; //3

    cout << s.rfind('d') << endl; //7
    cout << s.find('kkk') << endl; //-1 的十六进制 为 0xFFFFFFFF 十进制为 4294967295

    s.replace(2, 4, "AAA");
    cout << s << endl;
}

*/
/*
比较操作

compare函数在>时返回 1，<时返回 -1，==时返回 0。
比较区分大小写，比较时参考字典顺序，排越前面的越小。
大写的A比小写的a小。

int compare(const string&s) const;//与字符串s比较
int compare(const char *s) const;//与字符串s比较

*//*

void test06()
{
    string s1 = "hello";
    string s2 = "hello";
    const char* str = "world";

    if (s1.compare(s2) == 0)
    {
        cout << "s1 == s2 " << endl;
    }
    if (s2.compare(str) == 0)
    {
        cout << "s1 == str " << endl;
    }
    else
    {
        cout << "s2 != str" << endl;
    }
}

*/
/*
子串
string substr(int pos = 0, int n = npos) const;//返回由pos开始的n个字符组成的字符串

*//*

void test07()
{
    string email = "hello world@itcast.com";
    unsigned int pos = email.find('@');
    string username = email.substr(0, pos);
    cout << username << endl;

    string prex = email.substr(pos + 1);
    cout << prex << endl;
}

*/
/*
插入和删除操作
string& insert(int pos, const char* s); //插入字符串
string& insert(int pos, const string& str); //插入字符串
string& insert(int pos, int n, char c);//在指定位置插入n个字符c
string& erase(int pos, int n = npos);//删除从Pos开始的n个字符

*//*

void test08()
{
    string s = "aaaa";
    s.insert(3, "AAAA");
    cout << s << endl;

    s.insert(3, 5, 'M');
    cout << s << endl;

    s.erase(2, 3);
    cout << s << endl;

}
*/




int main()
{
   /* test01();
    test02();
    test03();
    test04();*/
    //test05();
    test06();
    test07();
    test08();
    test09();
    system("pause");
    return 0;
}

