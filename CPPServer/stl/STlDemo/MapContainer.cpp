//
// Created by Admin on 2021/11/18.
//
#include<iostream>
using namespace std;
#include<map>
#include<string>


/*
插入数据元素操作
map.insert(...); //往容器插入元素，返回pair<iterator,bool>
map<int, string> mapStu;
// 第一种 通过pair的方式插入对象
mapStu.insert(pair<int, string>(3, "小张"));
// 第二种 通过pair的方式插入对象
mapStu.inset(make_pair(-1, "校长"));
// 第三种 通过value_type的方式插入对象
mapStu.insert(map<int, string>::value_type(1, "小李"));
// 第四种 通过数组的方式插入值
mapStu[3] = "小刘";
mapStu[5] = "小王";

*/

template<class T>
void printMap(T &m)
{
    for(map<int, string>::iterator it = m.begin(); it != m.end(); ++it)
    {
        cout << "key:" << it->first << "value:" << it -> second << endl;
    }
}

void test01()
{
    map<int, string> myMap;

    //1.
    myMap.insert(pair<int, string>(3, "aaa"));

    //2.
    myMap.insert(make_pair(6, "bbb"));

    //3.
    myMap.insert(map<int,string>::value_type(2, "ccc"));

    // 4
    myMap[4] = "ddd";

    printMap(myMap);
}


/*
查找
find(key);//查找键key是否存在,若存在，返回该键的元素的迭代器；/若不存在，返回map.end();
count(keyElem);//返回容器中key为keyElem的对组个数。对map来说，要么是0，要么是1。对multimap来说，值可能大于1。
lower_bound(keyElem);//返回第一个key>=keyElem元素的迭代器。
upper_bound(keyElem);//返回第一个key>keyElem元素的迭代器。
equal_range(keyElem);//返回容器中key与keyElem相等的上下限的两个迭代器。

*/
void test04()
{
    map<int, string> myMap;
    myMap[1] = "aaa";
    myMap[2] = "bbb";
    myMap[3] = "ccc";
    myMap[4] = "ddd";
    myMap[5] = "eee";

    map<int,string>::iterator it = myMap.find(30);
    if(it == myMap.end())
        cout << "查找失败" << endl;
    else
        cout << "key:" << it->first << " value:" << it->second << endl;
    //查找大于等于3的最小的数
    it = myMap.upper_bound(3);
    if(it == myMap.end())
        cout << "查找失败" << endl;
    else
        cout << "key:" << it->first << " value:" << it->second << endl;
    //查找大于3的最小的数
    it = myMap.upper_bound(3);
    if(it == myMap.end())
        cout << "查找失败" << endl;
    else
        cout << "key:" << it->first << " value:" << it->second << endl;
}

int main()
{
    test04();
}