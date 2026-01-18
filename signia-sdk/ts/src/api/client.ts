
import axios from "axios";

export class SigniaClient {
  constructor(public baseUrl: string) {}

  async compile(input: unknown) {
    const res = await axios.post(`${this.baseUrl}/v1/compile`, input);
    return res.data;
  }

  async verify(payload: unknown) {
    const res = await axios.post(`${this.baseUrl}/v1/verify`, payload);
    return res.data;
  }
}
