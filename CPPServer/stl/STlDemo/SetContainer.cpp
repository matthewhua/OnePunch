//
// Created by Admin on 2021/11/18.
//

#include<iostream>

#include<set>//multset也是这个头文件
#include<algorithm>
#include<string>

using namespace std;

void test()
{
    set<int>::iterator it;
    it++;
    it--;
    //it + 2;err
    //双向迭代器
}

void printSet(set<int> &s)
{
    for (set<int>::iterator it = s.begin(); it != s.end(); ++it)
    {
        cout << *it << " ";
    }
    cout << endl;
}

void printSetFor(set<int> &s)
{
    for (auto &se : s){
        cout << se <<  endl;
    }

   // cout ;
}


/*
构造函数
set<T> st;//set默认构造函数：
mulitset<T> mst; //multiset默认构造函数:
set(const set &st);//拷贝构造函数

赋值操作
set&operator=(const set &st);//重载等号操作符
swap(st);//交换两个集合容器
大小操作
size();//返回容器中元素的数目
empty();//判断容器是否为空

插入和删除操作
insert(elem);//在容器中插入元素。
clear();//清除所有元素
erase(pos);//删除pos迭代器所指的元素，返回下一个元素的迭代器。
erase(beg, end);//删除区间[beg,end)的所有元素 ，返回下一个元素的迭代器。
erase(elem);//删除容器中值为elem的元素。

*/
void test01()
{
    set<int> s;
    s.insert(4);
    s.insert(8);
    s.insert(2);
    s.insert(10);
    s.insert(7);
    //自身规则进行排序, 从小到大
    printSetFor(s);
}


/*struct myfunc
{
    bool operator()(int v1, int v2)
    {
        return v1 > v2;
    }
};*//*

void printSet2(set<int, myfunc> &s)
{
    for(set<int, myfunc>::iterator it = s.begin(); it != s.end(); ++it)
    {
        cout << *it << " ";
    }
    cout << endl;
}

//改变set容器的规则，变为降序（从大到小）
void test02()
{
   // cout << "______________________________" << endl;
    set<int, myfunc> s;
    s.insert(4);
    s.insert(8);
    s.insert(2);
    s.insert(10);
    s.insert(7);

    printSet2(s);
}*/

void print(int v)
{
    cout << v << " ";
}

//set容器不允许有两个相同的元素
void test03()
{
    set<int> s;
    s.insert(4);
    s.insert(8);
    s.insert(2);
    s.insert(10);
    s.insert(7);

    printSetFor(s);
    //注意：set容器不允许有两个相同的元素，但是插入相同的元素，编译不会报错，运算也不报错
    //只是不把数据输入容器
    s.insert(8);
    printSetFor(s);

    pair<set<int>::iterator, bool> ret = s.insert(11);
    if (ret.second)
    {
        cout << "插入成功" << endl;
    }
    else
    {
        cout << "插入失败" << endl;
    }
    s.erase(2);
    s.erase(s.begin());
    printSetFor(s);


    //自身规则进行排序，从小到大
    //不能通过算法排序来排序关联式容器的元素
    //sort(s.begin(), s.end());

    for_each(s.begin(), s.end(), print);
}

/*
查找操作
find(key);//查找键key是否存在,若存在，返回该键的元素的迭代器；若不存在，返回set.end();
1.	count(key);//查找键key的元素个数
2.	lower_bound(keyElem);//返回第一个key>=keyElem元素的迭代器。
3.	upper_bound(keyElem);//返回第一个key>keyElem元素的迭代器。
equal_range(keyElem);//返回容器中key与keyElem相等的上下限的两个迭代器。

*/
void test05() {
    set<int> s;
    s.insert(4);
    s.insert(8);
    s.insert(6);
    s.insert(9);
    s.insert(7);

    auto iterator = s.find(10);
    if (iterator == s.end()){
        cout << "查找失败" << endl;
    }else{
        cout << "查找成功：" << *iterator << endl;
    }
    //查找大于等于2 的最小数
    iterator = s.lower_bound(2);

    //查找大于2的最小数
    iterator = s.upper_bound(2);
    if (iterator == s.end())
    {
        cout << "查找失败" << endl;
    }
    else
    {
        cout << "查找成功：" << *iterator << endl;
    }
    //返回大于等于2的两个最小的数，如果有2那么就返回2和大于2的最小数
    pair<set<int>::iterator, set<int>::iterator> ret = s.equal_range(4);
    cout << *(ret.first) << endl; //4
    cout << *(ret.second) << endl; //6 没有2 的话就返回和一 一样的值

    multiset<int> s1;
    s1.insert(44);
    s1.insert(44);
    s1.insert(44);
    s1.insert(9);
    s1.insert(7);
    cout << s1.count(44) << endl; // 3个

}


//存储对象时，需要告诉set容器规则
class Maker
{
public:
    Maker(string name,int age)
    {
        this->name = name;
        this->age = age;
    }
public:
    string name;
    int age;
};

/*struct MakerFunc
{
    bool operator<(const Maker &m1) const{
        return m1.age > this->;
    }
};

void test06()
{
    set<Maker, MakerFunc> s;
    s.insert(Maker("aaa", 18));
    s.insert(Maker("bbb", 19));
    s.insert(Maker("ccc", 20));
    s.insert(Maker("ddd", 21));
    s.insert(Maker("eee", 22));

    for (set<Maker, MakerFunc>::iterator it = s.begin(); it != s.end(); ++it)
    {
        cout << "Name:" << it->name << " Age:" << it->age << endl;
    }
}*/
int main()
{
   //test01();
    //test02();
    //    test03();
    test05();
}