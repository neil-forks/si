import { RelationMappings } from "objection";

import { User } from "@/datalayer/user";
import Model from "@/db";

export class Workspace extends Model {
  public readonly id!: number;
  public name!: string;
  public description?: string;
  public creator_id!: string;

  public creator!: User;
  public members!: User[];

  public static tableName = "workspaces";

  public static get relationMappings(): RelationMappings {
    return {
      creator: {
        relation: Model.BelongsToOneRelation,
        modelClass: User,
        join: {
          from: "workspaces.creator_id",
          to: "users.id",
        },
      },
      members: {
        relation: Model.ManyToManyRelation,
        modelClass: User,
        join: {
          from: "workspaces.id",
          through: {
            from: "users_workspaces.workspace_id",
            to: "users_workspaces.user_id",
          },
          to: "users.id",
        },
      },
    };
  }

  public static async deleteWorkspace(
    creator: User,
    id: number,
  ): Promise<Workspace> {
    let workspace = await Workspace.query()
      .where("id", id)
      .where("creator_id", creator.id)
      .first();
    await Workspace.query()
      .delete()
      .where("id", id)
      .where("creator_id", creator.id);
    return workspace;
  }

  public static async createWorkspace(
    wsName: string,
    creator: User,
    description: string,
  ): Promise<Workspace> {
    let workspace = await Workspace.query().insertAndFetch({
      //@ts-ignore This does exist, you bastards
      name: wsName,
      description: description,
      creator_id: creator.id, //eslint-disable-line
    });
    await workspace.$relatedQuery('members').relate(creator.id); //eslint-disable-line
    return workspace;
  }
}
