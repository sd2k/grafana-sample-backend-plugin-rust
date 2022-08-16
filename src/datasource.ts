import { Observable } from 'rxjs';

import { DataQueryRequest, DataQueryResponse, DataSourceInstanceSettings, LiveChannelScope } from '@grafana/data';
import { DataSourceWithBackend, getGrafanaLiveSrv, LiveDataStreamOptions, StreamingFrameAction } from '@grafana/runtime';

import { MyDataSourceOptions, MyQuery } from './types';

export class DataSource extends DataSourceWithBackend<MyQuery, MyDataSourceOptions> {
  constructor(instanceSettings: DataSourceInstanceSettings<MyDataSourceOptions>) {
    super(instanceSettings);
  }

  query(options: DataQueryRequest<MyQuery>): Observable<DataQueryResponse> {
    if (options.liveStreaming) {
      const options: LiveDataStreamOptions = {
        addr: {
          scope: LiveChannelScope.DataSource,
          // Replace with datasource UID.
          namespace: 'YlAe8PmVk',
          path: 'stream'
        },
        buffer: {
          action: StreamingFrameAction.Replace,
        },
      };
      return getGrafanaLiveSrv().getDataStream(options);
    }
    return super.query(options)
  }
}
