//
// Created by $Matthew on 2021-11-07.
//


#define _CRT_SECURE_NO_WARNINGS
#include<iostream>
#include<string>
using namespace std;

void test()
{
    string::iterator it;
    it++;
    it--;
    it + 2;
}

/*
���캯��
string();//����һ���յ��ַ��� ����: string str;
string(const string& str);//ʹ��һ��string�����ʼ����һ��string����
string(const char* s);//ʹ���ַ���s��ʼ��
string(int n, char c);//ʹ��n���ַ�c��ʼ��

*/
void test01()
{
    string s1;
    string s2(10, 'a');
    string s3(s2);
    string s4("hello");
}

/*
������ֵ����
string&operator=(const char* s);//char*�����ַ��� ��ֵ����ǰ���ַ���
string&operator=(const string&s);//���ַ���s������ǰ���ַ���
string&operator=(char c);//�ַ���ֵ����ǰ���ַ���
string& assign(const char *s);//���ַ���s������ǰ���ַ���
string& assign(const char *s, int n);//���ַ���s��ǰn���ַ�������ǰ���ַ���
string& assign(const string&s);//���ַ���s������ǰ�ַ���
string& assign(int n, char c);//��n���ַ�c������ǰ�ַ���
string& assign(const string&s, int start, int n);//��s��start��ʼn���ַ���ֵ���ַ���,��s=hello,��ôn=3,start=1����ô��hel�д�e��ʼ��ֵ3-1���ַ�

*/
void test02()
{
    string s1;
    s1 = "Hello";
    cout << s1 << endl;
    string s2;
    //s2.assign(s1);
    s2.assign("world");
    cout << s2 << endl;
}

/*
��ȡ�ַ�����
char&operator[](int n);//ͨ��[]��ʽȡ�ַ�
char& at(int n);//ͨ��at������ȡ�ַ�

*/
void test03()
{
    string s = "hello world";
    for (int i = 0; i < s.size(); i++) {
        cout << s[i] << " ";
    }
    cout << endl;
    for (int i = 0; i < s.size(); i++)
    {
        cout << s.at(i) << " ";
    }
    cout << endl;

    //[]��at������[]����Ԫ��ʱ��Խ�粻���쳣��ֱ�ӹң�atԽ�磬�����쳣
    try
    {
        //cout << s[100] << endl;
        cout << s.at(100) << endl;
    }
    catch (out_of_range &ex)
    {
        cout << ex.what() << endl;
        cout << "atԽ��" << endl;
    }
}

/*
ƴ�Ӳ���
string&operator+=(const string& str);//����+=������
string&operator+=(const char* str);//����+=������
string&operator+=(const char c);//����+=������
string& append(const char *s);//���ַ���s���ӵ���ǰ�ַ�����β
string& append(const char *s, int n);//���ַ���s��ǰn���ַ����ӵ���ǰ�ַ�����β
string& append(const string&s);//ͬoperator+=()
string& append(const string&s, int pos, int n);//���ַ���s�д�pos��ʼ��n���ַ����ӵ���ǰ�ַ�����β
string& append(int n, char c);//�ڵ�ǰ�ַ�����β���n���ַ�c

*/
void test04()
{
    string s1 = "aaa";
    s1 += "bbb";
    s1 += 'c';
    cout << s1 << endl;

    s1.append("ddddd", 3);
    cout << s1 << endl;
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

*/
void test05()
{
    string s = "abcdefgd";
    cout << s.find('d') << endl; // 3

    cout << s.rfind('d') << endl;//7
    cout << s.find("kkk") << endl; //-1 ��ʮ������ Ϊ 0xFFFFFFFF ʮ����Ϊ 4294967295

    s.replace(2, 4, "AAA");
    cout << s << endl;
}

/*
�Ƚϲ���

compare������>ʱ���� 1��<ʱ���� -1��==ʱ���� 0��
�Ƚ����ִ�Сд���Ƚ�ʱ�ο��ֵ�˳����Խǰ���ԽС��
��д��A��Сд��aС��

int compare(const string&s) const;//���ַ���s�Ƚ�
int compare(const char *s) const;//���ַ���s�Ƚ�

*/
void test06()
{
    string s1 = "hello";
    string s2 = "hello";
    const char* str = "world";
    if (s1.compare(s2) == 0)
    {
        cout << "s1==s2" << endl;
    }
    if (s1.compare(str) == 0)
    {
        cout << "s2==str" << endl;
    } else
    {
        cout << "s2 != str" << endl;
    }
}

/*
�Ӵ�
string substr(int pos = 0, int n = npos) const;//������pos��ʼ��n���ַ���ɵ��ַ���

*/
void test07()
{
    string email = "hello world@itcast.com";
    int pos = email.find('@');
    string username = email.substr(0, pos);
    cout << username << endl;

    string prex = email.substr(pos + 1);
    cout << prex << endl; //itcast.com

}

/*
�����ɾ������
string& insert(int pos, const char* s); //�����ַ���
string& insert(int pos, const string& str); //�����ַ���
string& insert(int pos, int n, char c);//��ָ��λ�ò���n���ַ�c
string& erase(int pos, int n = npos);//ɾ����Pos��ʼ��n���ַ�

*/
void test08()
{
    string s = "aaaa";
    s.insert(3, "AAA");
    cout << s << endl;

    s.insert(3, 5, 'M');
    cout << s << endl;

    s.erase(2, 3);
    cout << s << endl;
}


/*
string��c-style�ַ���ת��
*/
void test09()
{
    const char *str = "hello";
    string s = string(str);
    cout << s << endl;

    const char *str2 = s.c_str();
    cout << s << endl;


}

// �ַ����ڴ����·���, []��at��ȡ���ַ����ã������ܻ����
void test10()
{
    string s = "abcde";
    char &a = s[2];
    char &b = s[3];

    a = '1';
    b = '2';
    cout << "a:" << a << endl;
    cout << "b:" << b << endl;
    cout << s << endl;

    cout << "�ַ�����ԭ�ռ��ַ:" << (int*)s.c_str() << endl;

    s = "fdasfdasfdsafdasherewrkewhsaferew";
    cout << "�ַ����Ŀռ��ַ:" << (int*)s.c_str() << endl;


    //ԭ�ռ䱻�ͷţ�����a���Ǳ��ͷŵ�s[2]�ռ�ı�������������Ƿ��Ŀռ䣬�����
   // a = '3';
}

//�õ����������ַ���
void test11()
{
    string s = "hello";
    for (string::iterator i = s.begin(); i != s.end() ; ++i) {
        cout << *i << " ";
    }

    cout << endl;

    // �������
    for (string::reverse_iterator it = s.rbegin();  it != s.rend() ; ++it) {
        cout << *it << " ";
    }
    cout << endl;
}

int main(){
   /* test01();
    test02();
    test03();
    test04();
    test05();
    test06();*/
    test07();
    test08();
    test10();
    test11();
    return EXIT_FAILURE;
}