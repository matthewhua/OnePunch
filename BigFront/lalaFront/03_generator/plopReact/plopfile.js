// plopfile.js
module.exports = function (plop) {
   // 创建一个生成器
   plop.setGenerator('component', {
       description: '创建一个新的组件',
       prompts: [
           {
               type: 'input',
               name: 'name',
               message: '组件名称是什么？',
           },
       ],
       actions: [
           {
               type: 'add',
               path: 'src/components/{{pascalCase name}}/{{pascalCase name}}.js',
               templateFile: 'plop-templates/Component.js.hbs',
           },
           {
               type: 'add',
               path: 'src/components/{{pascalCase name}}/{{pascalCase name}}.css',
               templateFile: 'plop-templates/Component.css.hbs',
           },
       ],
   });
};