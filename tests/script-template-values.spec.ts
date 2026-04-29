import { expect, test, type Page } from '@playwright/test';

declare global {
  interface Window {
    __AUTODAILY_MOCK__?: {
      getState: () => {
        scriptTemplateValues: Record<
          string,
          { valuesJson: { variables?: Record<string, unknown>; taskSettings?: Record<string, unknown> } }
        >;
      };
      reset: () => unknown;
      seed: (partial: Record<string, unknown>) => unknown;
    };
  }
}

const seedTemplateValueState = async (page: Page) => {
  await page.goto('/time-templates');
  await page.evaluate(() => {
    if (!window.__AUTODAILY_MOCK__) {
      throw new Error('browser mock backend is not available');
    }

    const now = new Date().toISOString();
    window.__AUTODAILY_MOCK__.reset();
    window.__AUTODAILY_MOCK__.seed({
      devices: [
        {
          id: 'device-template-value',
          data: {
            deviceName: '模板设备',
            platform: 'android',
            exePath: null,
            exeArgs: null,
            cores: [],
            logLevel: 'Info',
            logToFile: false,
            adbConnect: null,
            capMethod: 'adb',
            imageCompression: 'ScreenCap',
            enable: true,
            autoStart: false,
            executionPolicy: {
              actionWaitMs: 0,
              progressTimeoutEnabled: false,
              progressTimeoutMs: 0,
              timeoutAction: 'notifyOnly',
              timeoutNotifyChannels: [],
            },
          },
        },
      ],
      scripts: [
        {
          id: 'script-template-value',
          data: {
            name: '模板变量脚本',
            description: null,
            userId: 'tester',
            userName: 'Tester',
            runtimeType: 'rhai',
            platform: 'android',
            sponsorshipQr: null,
            sponsorshipUrl: null,
            contactInfo: null,
            imgDetModel: null,
            txtDetModel: null,
            txtRecModel: null,
            pkgName: 'com.example.template',
            activityName: null,
            createTime: now,
            updateTime: now,
            verName: '1.0.0',
            verNum: 1,
            latestVer: 1,
            downloadCount: 0,
            scriptType: 'dev',
            isValid: true,
            allowClone: true,
            cloudId: null,
            runtimeSettings: { recoveryTaskId: null },
            variableCatalog: {
              version: 1,
              variables: [
                {
                  id: 'var-count',
                  key: 'input.count',
                  name: '次数',
                  namespace: 'input',
                  valueType: 'int',
                  ownerTaskId: 'task-template-value',
                  sourceType: 'manual',
                  sourceStepId: null,
                  readable: true,
                  writable: true,
                  persisted: true,
                  uiBindable: true,
                  defaultValue: 1,
                  description: '',
                },
                {
                  id: 'var-hidden-count',
                  key: 'input.hiddenCount',
                  name: '隐藏次数',
                  namespace: 'input',
                  valueType: 'int',
                  ownerTaskId: 'task-hidden-template-value',
                  sourceType: 'manual',
                  sourceStepId: null,
                  readable: true,
                  writable: true,
                  persisted: true,
                  uiBindable: true,
                  defaultValue: 1,
                  description: '',
                },
              ],
            },
          },
        },
      ],
      scriptTasks: {
        'script-template-value': [
          {
            id: 'task-template-value',
            scriptId: 'script-template-value',
            name: '日常任务',
            rowType: 'task',
            triggerMode: 'rootOnly',
            recordSchedule: true,
            sectionId: null,
            indentLevel: 0,
            defaultTaskCycle: 'everyRun',
            execMax: 0,
            showEnabledToggle: true,
            defaultEnabled: true,
            taskTone: 'normal',
            isHidden: false,
            data: {
              variables: { count: 1 },
              uiData: {
                layout: 'horizontal',
                fields: [
                  {
                    key: 'count',
                    label: '次数',
                    control: 'number',
                  },
                ],
              },
              steps: [],
            },
            createdAt: now,
            updatedAt: now,
            deletedAt: null,
            isDeleted: false,
            index: 0,
          },
          {
            id: 'task-hidden-template-value',
            scriptId: 'script-template-value',
            name: '隐藏任务',
            rowType: 'task',
            triggerMode: 'rootAndLink',
            recordSchedule: true,
            sectionId: null,
            indentLevel: 0,
            defaultTaskCycle: 'everyRun',
            execMax: 0,
            showEnabledToggle: true,
            defaultEnabled: true,
            taskTone: 'normal',
            isHidden: true,
            data: {
              variables: { hiddenCount: 1 },
              uiData: {
                layout: 'horizontal',
                fields: [
                  {
                    key: 'hiddenCount',
                    label: '隐藏次数',
                    control: 'number',
                    variableId: 'var-hidden-count',
                    inputKey: 'hiddenCount',
                  },
                ],
              },
              steps: [],
            },
            createdAt: now,
            updatedAt: now,
            deletedAt: null,
            isDeleted: false,
            index: 1,
          },
        ],
      },
      timeTemplates: [
        {
          id: 'template-morning',
          name: '早班',
          startTime: '08:00',
          endTime: '12:00',
        },
      ],
      assignmentsByDevice: {
        'device-template-value': [
          {
            id: 'assignment-template-value',
            deviceId: 'device-template-value',
            scriptId: 'script-template-value',
            timeTemplateId: 'template-morning',
            accountData: null,
            index: 0,
          },
        ],
      },
      scriptTemplateValues: {
        'device-template-value::script-template-value::template-morning::': {
          id: 'template-value-record',
          deviceId: 'device-template-value',
          scriptId: 'script-template-value',
          timeTemplateId: 'template-morning',
          accountId: null,
          valuesJson: {
            variables: {
              'var-count': 3,
              'var-hidden-count': 7,
            },
            taskSettings: {
              'task-hidden-template-value': { enabled: false, taskCycle: 'weekly' },
            },
          },
          createdAt: now,
          updatedAt: now,
        },
      },
    });
  });
  await page.reload();
};

test('saves template UI values bound by legacy field key', async ({ page }) => {
  await seedTemplateValueState(page);

  const countInput = page.getByTestId('editor-ui-preview-control-0');
  const enabledInput = page.getByTestId('editor-ui-preview-task-enabled');
  const cycleSelect = page.getByTestId('editor-ui-preview-task-cycle');
  await expect(page.getByText('隐藏任务')).not.toBeVisible();
  await expect(page.getByText('隐藏次数')).not.toBeVisible();
  await expect(page.getByText('1 个变量')).toBeVisible();
  await expect(countInput).toHaveValue('3');
  await expect(enabledInput).toBeChecked();
  await expect(cycleSelect).toContainText('每次运行');
  await countInput.fill('9');
  await enabledInput.uncheck();
  await cycleSelect.click();
  await page.getByTestId('editor-ui-preview-task-cycle-option-daily').click();
  await expect(enabledInput).not.toBeChecked();
  await page.getByRole('button', { name: '保存模板变量' }).click();

  await expect(countInput).toHaveValue('9');
  await expect(enabledInput).not.toBeChecked();
  await expect(cycleSelect).toContainText('每天');
  const state = await page.evaluate(() => window.__AUTODAILY_MOCK__?.getState());
  expect(
    state?.scriptTemplateValues['device-template-value::script-template-value::template-morning::'].valuesJson.variables?.['var-count'],
  ).toBe(9);
  expect(
    state?.scriptTemplateValues['device-template-value::script-template-value::template-morning::'].valuesJson.taskSettings?.['task-template-value'],
  ).toEqual({ enabled: false, taskCycle: 'daily' });
  expect(
    state?.scriptTemplateValues['device-template-value::script-template-value::template-morning::'].valuesJson.variables?.['var-hidden-count'],
  ).toBe(7);
  expect(
    state?.scriptTemplateValues['device-template-value::script-template-value::template-morning::'].valuesJson.taskSettings?.['task-hidden-template-value'],
  ).toEqual({ enabled: false, taskCycle: 'weekly' });
});

test('hides hidden script tasks from task management temporary picker', async ({ page }) => {
  await seedTemplateValueState(page);
  await page.goto('/tasks');

  await page.getByRole('button', { name: '临时运行' }).click();

  await expect(page.getByRole('button', { name: '模板变量脚本' })).toBeVisible();
  await expect(page.getByRole('button', { name: /日常任务 Task 运行/ })).toBeVisible();
  await expect(page.getByText('隐藏任务')).not.toBeVisible();
});
