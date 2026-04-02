import { expect, test, type Page } from '@playwright/test';
import type { ScriptTable } from '../src/types/bindings';

type StoredScriptTable = Omit<ScriptTable, 'data'> & {
  data: Omit<ScriptTable['data'], 'downloadCount' | 'latestVer' | 'verNum'> & {
    downloadCount: number;
    latestVer: number;
    verNum: number;
  };
};

const emptyVariableCatalog = {
  version: 1,
  variables: [],
} as const;

declare global {
  interface Window {
    __AUTODAILY_MOCK__?: {
      getState: () => {
        scripts: StoredScriptTable[];
      };
      reset: () => unknown;
    };
  }
}

const openCleanScriptsPage = async (page: Page) => {
  await page.goto('/scripts');
  await page.evaluate(() => {
    if (!window.__AUTODAILY_MOCK__) {
      throw new Error('browser mock backend is not available');
    }

    window.__AUTODAILY_MOCK__.reset();
  });
  await page.reload();
  await expect(page.getByRole('heading', { name: '本地脚本' })).toBeVisible();
};

const selectOptionByValue = async (page: Page, testId: string, value: string) => {
  await page.getByTestId(testId).click();
  await page.getByTestId(`${testId}-option-${value}`).click();
};

const getStoredScript = async (page: Page) => {
  const state = await page.evaluate(() => window.__AUTODAILY_MOCK__?.getState());
  expect(state?.scripts).toHaveLength(1);
  return state!.scripts[0];
};

test.beforeEach(async ({ page }) => {
  await openCleanScriptsPage(page);
});

test('disables submit when script name is blank', async ({ page }) => {
  await page.getByTestId('script-list-create-button').click();

  const dialog = page.getByRole('dialog', { name: '新建脚本' });
  const nameInput = dialog.getByTestId('script-basic-name');
  const submitButton = dialog.getByTestId('script-submit');

  await expect(nameInput).toHaveValue('未命名脚本 1');
  await nameInput.fill('');

  await expect(submitButton).toBeDisabled();
});

test('creates a local script with basic, model, and sponsorship settings', async ({ page }) => {
  const scriptName = `每日清体力 ${Date.now()}`;
  const description = '覆盖基本信息、模型信息和赞助信息的创建流程';
  const packageName = 'com.example.daily';
  const qrDataUrl =
    'data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVQIW2P8//8/AwAI/AL+X2HFVQAAAABJRU5ErkJggg==';

  await page.getByTestId('script-list-create-button').click();

  const dialog = page.getByRole('dialog', { name: '新建脚本' });
  await expect(dialog).toBeVisible();

  await dialog.getByTestId('script-basic-name').fill(scriptName);
  await dialog.getByTestId('script-basic-description').fill(description);
  await selectOptionByValue(page, 'script-basic-runtime-type', 'javaScript');
  await dialog.getByTestId('script-basic-package-name').fill(packageName);
  await dialog.getByTestId('script-basic-version-name').fill('2.0.0');
  await dialog.getByTestId('script-basic-version-num').fill('7');
  await dialog.getByTestId('script-basic-allow-clone').uncheck();

  await dialog.getByTestId('script-dialog-tab-models').click();

  await selectOptionByValue(page, 'script-models-img-det-kind', 'Yolo11');
  await selectOptionByValue(page, 'script-models-img-det-base-model-source', 'Custom');
  await selectOptionByValue(page, 'script-models-img-det-base-execution-provider', 'DirectML');
  await dialog.getByTestId('script-models-img-det-base-model-path').fill('D:\\models\\img-det.onnx');
  await dialog.getByTestId('script-models-img-det-class-count').fill('3');
  await dialog.getByTestId('script-models-img-det-label-path').fill('D:\\models\\img-det.labels.yaml');
  await dialog.getByTestId('script-models-img-det-confidence').fill('0.55');
  await dialog.getByTestId('script-models-img-det-iou').fill('0.35');

  await selectOptionByValue(page, 'script-models-txt-det-kind', 'PaddleDbNet');
  await selectOptionByValue(page, 'script-models-txt-det-base-model-source', 'Custom');
  await dialog.getByTestId('script-models-txt-det-base-model-path').fill('D:\\models\\txt-det.onnx');
  await dialog.getByTestId('script-models-txt-det-db-thresh').fill('0.42');
  await dialog.getByTestId('script-models-txt-det-db-box-thresh').fill('0.66');
  await dialog.getByTestId('script-models-txt-det-unclip-ratio').fill('2.4');
  await dialog.getByTestId('script-models-txt-det-use-dilation').check();

  await selectOptionByValue(page, 'script-models-txt-rec-kind', 'PaddleCrnn');
  await selectOptionByValue(page, 'script-models-txt-rec-base-model-source', 'Custom');
  await dialog.getByTestId('script-models-txt-rec-base-model-path').fill('D:\\models\\txt-rec.onnx');
  await dialog.getByTestId('script-models-txt-rec-dict-path').fill('D:\\models\\keys.txt');
  await selectOptionByValue(page, 'script-models-txt-rec-resize-filter', 'Triangle');
  await selectOptionByValue(page, 'script-models-txt-rec-processing-mode', 'MicroBatch');
  await dialog.getByTestId('script-models-txt-rec-micro-batch-size').fill('3');
  await dialog.getByTestId('script-models-txt-rec-width-bucket-step').fill('24');

  await dialog.getByTestId('script-dialog-tab-support').click();
  await dialog.getByTestId('script-support-contact-info').fill('tester@example.com');
  await dialog.getByTestId('script-support-sponsorship-url').fill('https://example.com/sponsor');
  await dialog.getByTestId('script-support-sponsorship-qr-input').fill(qrDataUrl);
  await expect(dialog.getByTestId('script-support-sponsorship-qr-source')).toContainText('Data URL');

  await dialog.getByTestId('script-submit').click();
  await expect(dialog).not.toBeVisible();
  await expect(page.getByRole('heading', { name: scriptName })).toBeVisible();

  const script = await getStoredScript(page);
  expect(script.data.name).toBe(scriptName);
  expect(script.data.description).toBe(description);
  expect(script.data.runtimeType).toBe('javaScript');
  expect(script.data.pkgName).toBe(packageName);
  expect(script.data.verName).toBe('2.0.0');
  expect(script.data.verNum).toBe(7);
  expect(script.data.allowClone).toBe(false);
  expect(script.data.variableCatalog).toEqual(emptyVariableCatalog);

  expect(script.data.imgDetModel).toEqual({
    Yolo11: expect.objectContaining({
      classCount: 3,
      confidenceThresh: 0.55,
      iouThresh: 0.35,
      labelPath: 'D:\\models\\img-det.labels.yaml',
      baseModel: expect.objectContaining({
        modelSource: 'Custom',
        executionProvider: 'DirectML',
        modelPath: 'D:\\models\\img-det.onnx',
      }),
    }),
  });

  expect(script.data.txtDetModel).toEqual({
    PaddleDbNet: expect.objectContaining({
      dbThresh: 0.42,
      dbBoxThresh: 0.66,
      unclipRatio: 2.4,
      useDilation: true,
      baseModel: expect.objectContaining({
        modelSource: 'Custom',
        modelPath: 'D:\\models\\txt-det.onnx',
      }),
    }),
  });

  expect(script.data.txtRecModel).toEqual({
    PaddleCrnn: expect.objectContaining({
      dictPath: 'D:\\models\\keys.txt',
      resizeFilter: 'Triangle',
      processingMode: 'MicroBatch',
      microBatchSize: 3,
      widthBucketStep: 24,
      baseModel: expect.objectContaining({
        modelSource: 'Custom',
        modelPath: 'D:\\models\\txt-rec.onnx',
      }),
    }),
  });

  expect(script.data.contactInfo).toBe('tester@example.com');
  expect(script.data.sponsorshipUrl).toBe('https://example.com/sponsor');
  expect(script.data.sponsorshipQr).toBe(qrDataUrl);

  await page.reload();
  await expect(page.getByRole('heading', { name: scriptName })).toBeVisible();
});

test('creates a local script with yolo26 detector settings', async ({ page }) => {
  const scriptName = `YOLO26 检测 ${Date.now()}`;

  await page.getByTestId('script-list-create-button').click();

  const dialog = page.getByRole('dialog', { name: '新建脚本' });
  await dialog.getByTestId('script-basic-name').fill(scriptName);
  await dialog.getByTestId('script-dialog-tab-models').click();

  await selectOptionByValue(page, 'script-models-img-det-kind', 'Yolo26');
  await selectOptionByValue(page, 'script-models-img-det-base-model-source', 'Custom');
  await dialog.getByTestId('script-models-img-det-base-model-path').fill('D:\\models\\img-det-yolo26.onnx');
  await dialog.getByTestId('script-models-img-det-class-count').fill('5');
  await dialog.getByTestId('script-models-img-det-label-path').fill('D:\\models\\img-det-yolo26.labels.yaml');
  await dialog.getByTestId('script-models-img-det-confidence').fill('0.4');
  await dialog.getByTestId('script-models-img-det-iou').fill('0.2');

  await selectOptionByValue(page, 'script-models-txt-det-kind', 'Yolo26');
  await selectOptionByValue(page, 'script-models-txt-det-base-model-source', 'Custom');
  await dialog.getByTestId('script-models-txt-det-base-model-path').fill('D:\\models\\txt-det-yolo26.onnx');
  await dialog.getByTestId('script-models-txt-det-class-count').fill('2');
  await dialog.getByTestId('script-models-txt-det-confidence').fill('0.35');
  await dialog.getByTestId('script-models-txt-det-iou').fill('0.15');

  await dialog.getByTestId('script-submit').click();
  await expect(dialog).not.toBeVisible();

  const script = await getStoredScript(page);
  expect(script.data.imgDetModel).toEqual({
    Yolo26: expect.objectContaining({
      classCount: 5,
      confidenceThresh: 0.4,
      iouThresh: 0.2,
      labelPath: 'D:\\models\\img-det-yolo26.labels.yaml',
      baseModel: expect.objectContaining({
        modelPath: 'D:\\models\\img-det-yolo26.onnx',
        modelType: 'Yolo26',
      }),
    }),
  });

  expect(script.data.txtDetModel).toEqual({
    Yolo26: expect.objectContaining({
      classCount: 2,
      confidenceThresh: 0.35,
      iouThresh: 0.15,
      txtIdx: 0,
      baseModel: expect.objectContaining({
        modelPath: 'D:\\models\\txt-det-yolo26.onnx',
        modelType: 'Yolo26',
      }),
    }),
  });
});
