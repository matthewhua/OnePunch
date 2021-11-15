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

/*
���Һ��滻
int find(const string& str, int pos = 0) const; //����str��һ�γ���λ��,��pos��ʼ����
int find(constchar* s, int pos = 0) const;  //����s��һ�γ���λ��,��pos��ʼ����
int find(constchar* s, int pos, int n) const;  //��posλ�ò���s��ǰn���ַ���һ��λ��
int find(constchar c, int pos = 0) const;  //�����ַ�c��һ�γ���λ��
int rfind(conststring& str, int pos = npos) const;//����str���һ��λ��,��pos��ʼ����
int rfind(constchar* s, int pos = npos) const;//����s���һ�γ���λ��,��pos��ʼ����
int rfind(constchar* s, int pos, int n) const;//��pos����s��ǰn���ַ����һ��λ��
int rfind(constchar c, int pos = 0) const; //�����ַ�c���һ�γ���λ��
string& replace(int pos, int n, const string& str); //�滻��pos��ʼn���ַ�Ϊ�ַ���str
string& replace(int pos, int n, const char* s); //�滻��pos��ʼ��n���ַ�Ϊ�ַ���s

*//*

void test05()
{
    string s = "abcdefgd";
    cout << s.find('d') << endl; //3

    cout << s.rfind('d') << endl; //7
    cout << s.find('kkk') << endl; //-1 ��ʮ������ Ϊ 0xFFFFFFFF ʮ����Ϊ 4294967295

    s.replace(2, 4, "AAA");
    cout << s << endl;
}

*/
/*
�Ƚϲ���

compare������>ʱ���� 1��<ʱ���� -1��==ʱ���� 0��
�Ƚ����ִ�Сд���Ƚ�ʱ�ο��ֵ�˳����Խǰ���ԽС��
��д��A��Сд��aС��

int compare(const string&s) const;//���ַ���s�Ƚ�
int compare(const char *s) const;//���ַ���s�Ƚ�

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
�Ӵ�
string substr(int pos = 0, int n = npos) const;//������pos��ʼ��n���ַ���ɵ��ַ���

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
�����ɾ������
string& insert(int pos, const char* s); //�����ַ���
string& insert(int pos, const string& str); //�����ַ���
string& insert(int pos, int n, char c);//��ָ��λ�ò���n���ַ�c
string& erase(int pos, int n = npos);//ɾ����Pos��ʼ��n���ַ�

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

