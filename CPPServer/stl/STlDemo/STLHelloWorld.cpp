//
// Created by $Matthew on 2021-11-07.
//
#define _CRT_SECURE_NO_WARNINGS
#include<string>
#include <vector> // ����
#include <algorithm> //�㷨
#include<iostream>
using namespace std;


//�����㷨�Ļص�����
void MyPrint(int val)
{
    cout << val << " ";
}
// 1. �洢������������
void test01()
{
    // ����
    vector<int> v;
    v.push_back(10);
    v.push_back(20);
    v.push_back(30);
    v.push_back(40);
    v.push_back(50);

    //��ȡ��ʼλ�õĵ�����
    vector<int>::iterator  begin = v.begin();
    //��ȡ����λ�õĵ�����
    vector<int>::iterator end= v.end();
    //�����㷨
    for_each(begin, end, MyPrint);
    cout << endl;
}

// 2. �����洢����
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
    //�������д洢����
    v.push_back(Maker("���",  18));
    v.push_back(Maker("С��", 19));

    v.push_back(Maker("�̵�", 180));

    //��ȡ��ʼ�ͽ���λ�õĵ�����
    vector<Maker>::iterator begin = v.begin();
    vector<Maker>::iterator end = v.end();
    while (begin != end)
    {
        cout << (*begin);
        begin++;
    }
}

//3. �洢�����ָ��
void test03()
{
    vector<Maker*> v;
    //��������
    Maker *m1 = new Maker("���", 18);
    Maker *m2 = new Maker("С��", 19);
    Maker *m3 = new Maker("������",200 );
    Maker *m4 = new Maker("������",180 );
    Maker *m5 = new Maker("�̵�", 18);

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

// 4. ����Ƕ������
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
int main()
{
    test01();
    test02();
    test03();
    test04();
    system("pause");
    return 0;
}