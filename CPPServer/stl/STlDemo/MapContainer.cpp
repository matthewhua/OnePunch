//
// Created by Admin on 2021/11/18.
//
#include<iostream>
using namespace std;
#include<map>
#include<string>


/*
��������Ԫ�ز���
map.insert(...); //����������Ԫ�أ�����pair<iterator,bool>
map<int, string> mapStu;
// ��һ�� ͨ��pair�ķ�ʽ�������
mapStu.insert(pair<int, string>(3, "С��"));
// �ڶ��� ͨ��pair�ķ�ʽ�������
mapStu.inset(make_pair(-1, "У��"));
// ������ ͨ��value_type�ķ�ʽ�������
mapStu.insert(map<int, string>::value_type(1, "С��"));
// ������ ͨ������ķ�ʽ����ֵ
mapStu[3] = "С��";
mapStu[5] = "С��";

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
����
find(key);//���Ҽ�key�Ƿ����,�����ڣ����ظü���Ԫ�صĵ�������/�������ڣ�����map.end();
count(keyElem);//����������keyΪkeyElem�Ķ����������map��˵��Ҫô��0��Ҫô��1����multimap��˵��ֵ���ܴ���1��
lower_bound(keyElem);//���ص�һ��key>=keyElemԪ�صĵ�������
upper_bound(keyElem);//���ص�һ��key>keyElemԪ�صĵ�������
equal_range(keyElem);//����������key��keyElem��ȵ������޵�������������

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
        cout << "����ʧ��" << endl;
    else
        cout << "key:" << it->first << " value:" << it->second << endl;
    //���Ҵ��ڵ���3����С����
    it = myMap.upper_bound(3);
    if(it == myMap.end())
        cout << "����ʧ��" << endl;
    else
        cout << "key:" << it->first << " value:" << it->second << endl;
    //���Ҵ���3����С����
    it = myMap.upper_bound(3);
    if(it == myMap.end())
        cout << "����ʧ��" << endl;
    else
        cout << "key:" << it->first << " value:" << it->second << endl;
}

int main()
{
    test04();
}