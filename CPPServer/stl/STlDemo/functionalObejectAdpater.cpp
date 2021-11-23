#include<iostream>

#include<algorithm>
#include<functional>
#include<vector>
#include<string>
using namespace std;

struct Myfunc:public binary_function<int, int, void>
{
    void operator()(int v1, int v2)const
    {
        cout << "v1=" << v1 << endl;
        cout << "v2=" << v2 << endl;
        cout << v1 + v2 << endl;//第三步：实现函数体
    }
};

//需求：打印时，每个元素加100然后打印出来
void test()
{
	vector<int> v;
	v.push_back(10);
	v.push_back(20);
	v.push_back(30);
	v.push_back(40);
	v.push_back(50);
//第四步：用bind2nd来绑定函数对象
	for_each(v.begin(), v.end(), bind2nd(Myfunc(),100));
    //for_each(v.begin() ,v.end(), Myfunc())
}
//2.bind1st和bind2nd的区别
//bind1st把100绑定给第一个参数
//bind2nd把100绑定给第二个参数

//第一步： 继承
struct MyNotfunc:public unary_function<int, bool>
{
    bool operator()(int v) const
    {
        return v >= 20;
    }
};

void myPrint(int val)
{
    cout << val << " ";
}

void test02()
{
    vector<int> v;
	v.push_back(10);
	v.push_back(50);
	v.push_back(30);
	v.push_back(40);
	v.push_back(50);

    //第三步：适配
    vector<int>::iterator it = find_if(v.begin(), v.end(), not1(MyNotfunc()));
    	if (it == v.end())
	{
		cout << "查找失败" << endl;
	}
	else
	{
		cout << "查找成功=" << *it << endl;
	}
}

//普通函数进行适配 ptr_fun
//第一步:把一个参数变为二个参数
void myprint2(int val,int val2)
{
	cout << val+val2<< " ";
}



void test03()
{
    vector<int> v;
    v.push_back(10);
    v.push_back(11);
    v.push_back(30);



	v.push_back(50);
    //第二步：把普通函数变为函数对象 ptr_fun
    for_each(v.begin(), v.end(), bind2nd(ptr_fun(myprint2), 100));
}

class Maker
{
public:
    Maker(const string &name, int age) : name(name), age(age) {}
//成员函数
	void MyprintMaker()
	{
		cout << "Name:" << this->name << " Age:" << this->age << endl;
	}
public:
    string name;
    int age;
};
//void MyprintMaker(Maker m)
//{
//	cout << "Name:" << m.name << " Age:" << m.age << endl;
//}

void test04()
{
    vector<Maker> v;
    v.push_back(Maker("aaa", 10));
    v.push_back(Maker("bbb", 20));
	v.push_back(Maker("ccc", 30));
	// 当对象存储的啥对象， 用mem_fun_ref适配他的成员函数
    for_each(v.begin(), v.end(), mem_fun_ref(&Maker::MyprintMaker));
    cout << "---------------" << endl;
    vector<Maker*> vp;
	vp.push_back(new Maker("aaa", 10));
	vp.push_back(new Maker("bbb", 20));
	vp.push_back(new Maker("ccc", 30));
    //当容器存储的是对象指针，用mem_fun适配他的成员函数
    for_each(vp.begin(), vp.end(), mem_fn(&Maker::MyprintMaker));
}




int main()
{
    //test();
    //test02();
    test03();
    return EXIT_SUCCESS;
}

