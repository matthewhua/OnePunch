//
// Created by Admin on 2021/11/15.
//

#include<iostream>
using namespace std;
#include<list>
#include <vector>
#include <algorithm>

void test()
{
    list<int>::iterator it;
    it--;
    it++;
    //it + 2; err
    //˫�������
    list<int> mylist;
    for (int i = 0; i < 10; ++i) {
        mylist.push_back(i);
    }

    //2015��2017vs :_Myhead==>_Myhead(),_Mysize==>_Mysize()


}

void printList(const list<int> &myList)
{
    for (auto it=  myList.begin(); it != myList.end() ; ++it) {
        cout << *it << " ";
    }
    cout << endl;
}


/**
����
list<T> lstT;//list���ò���ģ����ʵ��,�����Ĭ�Ϲ�����ʽ��
list(beg,end);//���캯����[beg, end)�����е�Ԫ�ؿ���������
list(n,elem);//���캯����n��elem����������
list(const list &lst);//�������캯����

*/
void test01()
{
    list<int> myList1(10, 6); // 10��
    cout << myList1.size() << endl;
    list<int> myList2(++myList1.begin(), --myList1.end()); //8��
    cout << myList2.size() << endl;
    printList(myList2);
}

bool myFunc(int val)
{
    return val > 300;
}


/**
����Ԫ�ز����ɾ������
push_back(elem);//������β������һ��Ԫ��
pop_back();//ɾ�����������һ��Ԫ��
push_front(elem);//��������ͷ����һ��Ԫ��
pop_front();//��������ͷ�Ƴ���һ��Ԫ��
insert(pos,elem);//��posλ�ò�elemԪ�صĿ��������������ݵ�λ�á�
insert(pos,n,elem);//��posλ�ò���n��elem���ݣ��޷���ֵ��
insert(pos,beg,end);//��posλ�ò���[beg,end)��������ݣ��޷���ֵ��
clear();//�Ƴ���������������
erase(beg,end);//ɾ��[beg,end)��������ݣ�������һ�����ݵ�λ�á�
erase(pos);//ɾ��posλ�õ����ݣ�������һ�����ݵ�λ�á�
remove(elem);//ɾ��������������elemֵƥ���Ԫ�ء�

*/
void test02()
{
    list<int> myList;
    myList.push_back(10);
    myList.push_back(20);
    myList.push_back(30);
    myList.push_back(40);
    myList.push_back(50);
    myList.push_front(100);
    myList.push_front(200);
    myList.push_front(300);
    myList.push_front(400);

    vector<int> v;
    v.push_back(1000);
    v.push_back(2000);
    v.push_back(3000);
    myList.insert(myList.begin(), v.begin(), v.end());

    printList(myList);

    //Ҫɾ������300 ������
    myList.remove_if(myFunc);
    printList(myList);
}


/**
��С����
size();//����������Ԫ�صĸ���
empty();//�ж������Ƿ�Ϊ��
resize(num);//����ָ�������ĳ���Ϊnum��
�������䳤������Ĭ��ֵ�����λ�á�
���������̣���ĩβ�����������ȵ�Ԫ�ر�ɾ����
resize(num, elem);//����ָ�������ĳ���Ϊnum��
�������䳤������elemֵ�����λ�á�
���������̣���ĩβ�����������ȵ�Ԫ�ر�ɾ����

*/
void test03()
{
    list<int> myList;
    for (int i = 0; i < 5; ++i) {
        myList.push_back(i + 1);
    }

    cout << "size:" << myList.size() << endl;
    cout << myList.empty() << endl;
    if (myList.empty())
    {
        cout << "��"<< endl;
    }
    else
    {
        cout << "��Ϊ��" << endl;
    }

    myList.resize(3);
    printList(myList);
}


/**
��ֵ����,���ݵĴ�ȡ
assign(beg, end);//��[beg, end)�����е����ݿ�����ֵ������
assign(n, elem);//��n��elem������ֵ������
list&operator=(const list &lst);//���صȺŲ�����
swap(lst);//��lst�뱾���Ԫ�ػ�����

front();//���ص�һ��Ԫ�ء�
back();//�������һ��Ԫ�ء�

*/
void test04()
{
    list<int> mylist;
    mylist.assign(10, 10);
    printList(mylist);

    cout << mylist.front() << endl;
    cout << mylist.back() << endl;

    list<int> myList2;
    for (int i = 0; i < 5; i++)
    {
        myList2.push_back(i);
    }
    printList(myList2);
    myList2.swap(mylist);
    printList(myList2);

 }


/*
��ת ����
reverse();//��ת��������lst����1,3,5Ԫ�أ����д˷�����lst�Ͱ���5,3,1Ԫ�ء�
sort(); //list����

*/

bool myfunc2(int v1, int v2)
{
    return v1 > v2;
}

void test05()
{
    list<int> mylist;
    for (int i = 0; i < 5; i++)
    {
        mylist.push_back(i + 10);
    }

    printList(mylist);

    mylist.reverse();
    printList(mylist);
    //ע�⣺list��������ʹ��sort�㷨
    //sort(mylist.begin(), mylist.end());

    mylist.sort();
    printList(mylist);

    mylist.sort(myfunc2);
    printList(mylist);
}

int main()
{
 /*   //test();
    //test01();
    test02();
    test03();
    test04();*/
    test05();

    system("pause");
    return EXIT_SUCCESS;
}

