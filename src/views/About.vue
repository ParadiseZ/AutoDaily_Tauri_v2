<template>
  <div class="about-page">
    <div class="about-container">
      <div class="logo-section">
        <img src="../assets/logo.png" alt="AutoDaily Logo" class="logo" />
        <h1>AutoDaily</h1>
        <div class="version">版本 1.0.0</div>
      </div>
      
      <div class="description-section">
        <p class="app-description">
          AutoDaily是一款功能强大的自动化工具，专为日常任务自动化而设计。它集成了OCR识别、YOLO目标检测和模式匹配等多种技术，帮助您轻松实现各种自动化需求。
        </p>
      </div>
      
      <el-divider />
      
      <el-collapse>
        <el-collapse-item title="系统信息" name="system-info">
          <el-descriptions :column="1" border>
            <el-descriptions-item label="操作系统">{{ systemInfo.os }}</el-descriptions-item>
            <el-descriptions-item label="设备">{{ systemInfo.device }}</el-descriptions-item>
            <el-descriptions-item label="CPU">{{ systemInfo.cpu }}</el-descriptions-item>
            <el-descriptions-item label="内存">{{ systemInfo.memory }}</el-descriptions-item>
            <el-descriptions-item label="GPU">{{ systemInfo.gpu }}</el-descriptions-item>
          </el-descriptions>
        </el-collapse-item>
        
        <el-collapse-item title="特性与功能" name="features">
          <div class="features-list">
            <div v-for="(feature, index) in features" :key="index" class="feature-item">
              <h3>{{ feature.title }}</h3>
              <p>{{ feature.description }}</p>
            </div>
          </div>
        </el-collapse-item>
        
        <el-collapse-item title="技术栈" name="tech-stack">
          <div class="tech-stack">
            <h3>前端</h3>
            <div class="tech-items">
              <el-tag v-for="tech in techStack.frontend" :key="tech" class="tech-tag">{{ tech }}</el-tag>
            </div>
            
            <h3>后端</h3>
            <div class="tech-items">
              <el-tag v-for="tech in techStack.backend" :key="tech" type="success" class="tech-tag">{{ tech }}</el-tag>
            </div>
            
            <h3>机器学习</h3>
            <div class="tech-items">
              <el-tag v-for="tech in techStack.ml" :key="tech" type="warning" class="tech-tag">{{ tech }}</el-tag>
            </div>
          </div>
        </el-collapse-item>
        
        <el-collapse-item title="更新日志" name="changelog">
          <div class="changelog">
            <div v-for="(log, index) in changelog" :key="index" class="changelog-entry">
              <div class="version-info">
                <h3>{{ log.version }}</h3>
                <span class="release-date">{{ log.date }}</span>
              </div>
              <ul class="changes-list">
                <li v-for="(change, cidx) in log.changes" :key="cidx">{{ change }}</li>
              </ul>
            </div>
          </div>
        </el-collapse-item>
        
        <el-collapse-item title="开源许可" name="license">
          <div class="license-section">
            <p>本软件基于 <strong>MIT 许可证</strong> 发布</p>
            <pre class="license-text">
MIT License

Copyright (c) 2024 AutoDaily

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
            </pre>
          </div>
        </el-collapse-item>
      </el-collapse>
      
      <div class="contact-section">
        <h3>联系我们</h3>
        <div class="contact-links">
          <el-button round>
            <el-icon><message /></el-icon>
            官方论坛
          </el-button>
          <el-button round>
            <el-icon><document /></el-icon>
            查看文档
          </el-button>
          <el-button round>
            <el-icon><position /></el-icon>
            问题反馈
          </el-button>
        </div>
      </div>
      
      <div class="footer">
        <p>Copyright © 2024 AutoDaily. All rights reserved.</p>
      </div>
    </div>
  </div>
</template>

<script setup>
import { reactive } from 'vue';
import { Message, Document, Position } from '@element-plus/icons-vue';

// 系统信息
const systemInfo = reactive({
  os: 'Windows 10 Pro',
  device: 'PC',
  cpu: 'Intel Core i7-10700K',
  memory: '16GB',
  gpu: 'NVIDIA GeForce RTX 3060'
});

// 特性与功能
const features = [
  {
    title: '高效的OCR识别',
    description: '集成多种OCR引擎，可以精准识别各种场景下的文本信息。'
  },
  {
    title: 'YOLO目标检测',
    description: '基于最新的YOLO算法，可以准确识别和定位各种物体和目标。'
  },
  {
    title: '脚本市场',
    description: '丰富的脚本市场，可以下载和分享各种自动化脚本。'
  },
  {
    title: '多设备支持',
    description: '支持通过ADB连接安卓设备，同时支持对窗口进行操作。'
  },
  {
    title: '智能决策引擎',
    description: '基于规则和机器学习的决策引擎，可以根据场景自动选择最佳操作。'
  }
];

// 技术栈
const techStack = {
  frontend: ['Vue 3', 'Element Plus', 'Vite', 'SCSS', 'TypeScript'],
  backend: ['Rust', 'Tauri', 'SQLite', 'C++'],
  ml: ['YOLO', 'PaddleOCR', 'OpenCV', 'ONNX Runtime']
};

// 更新日志
const changelog = [
  {
    version: 'v1.0.0',
    date: '2024-05-20',
    changes: [
      '首次发布',
      '支持ADB和窗口截图',
      'OCR和YOLO检测支持',
      '脚本市场功能',
      '基本的设置和配置功能'
    ]
  },
  {
    version: 'v0.9.5 Beta',
    date: '2024-04-15',
    changes: [
      '添加了多设备管理',
      '改进OCR识别准确率',
      '优化内存使用',
      '修复了多项UI错误'
    ]
  },
  {
    version: 'v0.8.0 Alpha',
    date: '2024-03-10',
    changes: [
      '初始测试版本',
      '基本框架搭建',
      'OCR初步集成',
      '简单的脚本运行功能'
    ]
  }
];
</script>

<style lang="scss" scoped>
.about-page {
  display: flex;
  justify-content: center;
  padding: 20px;
}

.about-container {
  max-width: 800px;
  width: 100%;
}

.logo-section {
  display: flex;
  flex-direction: column;
  align-items: center;
  margin-bottom: 30px;
  
  .logo {
    width: 120px;
    height: 120px;
    margin-bottom: 10px;
  }
  
  h1 {
    margin: 10px 0 0;
    font-size: 32px;
    background: var(--primary-gradient);
    -webkit-background-clip: text;
    -webkit-text-fill-color: transparent;
  }
  
  .version {
    color: var(--text-color-secondary);
    margin-top: 5px;
  }
}

.description-section {
  text-align: center;
  margin-bottom: 30px;
  
  .app-description {
    font-size: 16px;
    line-height: 1.6;
    color: var(--text-color-regular);
  }
}

.features-list {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
  gap: 20px;
  
  .feature-item {
    h3 {
      margin-top: 0;
      margin-bottom: 10px;
      font-size: 16px;
      color: var(--el-color-primary);
    }
    
    p {
      margin: 0;
      color: var(--text-color-regular);
    }
  }
}

.tech-stack {
  h3 {
    margin-top: 15px;
    margin-bottom: 10px;
    font-size: 16px;
  }
  
  .tech-items {
    display: flex;
    flex-wrap: wrap;
    gap: 10px;
    margin-bottom: 15px;
  }
  
  .tech-tag {
    margin-right: 5px;
  }
}

.changelog {
  .changelog-entry {
    margin-bottom: 20px;
    
    &:last-child {
      margin-bottom: 0;
    }
    
    .version-info {
      display: flex;
      align-items: center;
      gap: 10px;
      margin-bottom: 8px;
      
      h3 {
        margin: 0;
        font-size: 16px;
      }
      
      .release-date {
        color: var(--text-color-secondary);
        font-size: 14px;
      }
    }
    
    .changes-list {
      margin: 0;
      padding-left: 20px;
      
      li {
        margin-bottom: 5px;
      }
    }
  }
}

.license-section {
  .license-text {
    background-color: var(--bg-color-soft);
    padding: 15px;
    border-radius: 5px;
    font-size: 12px;
    white-space: pre-wrap;
    overflow-x: auto;
  }
}

.contact-section {
  margin-top: 30px;
  text-align: center;
  
  h3 {
    font-size: 18px;
    margin-bottom: 15px;
  }
  
  .contact-links {
    display: flex;
    justify-content: center;
    gap: 15px;
    flex-wrap: wrap;
  }
}

.footer {
  margin-top: 40px;
  text-align: center;
  color: var(--text-color-secondary);
  font-size: 14px;
}
</style> 