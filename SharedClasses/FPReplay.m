//
//  FPReplay.m
//  IronJumpLevelEditor
//
//  Created by Filip Kunc on 8/7/10.
//  Copyright (c) 2010 Filip Kunc. All rights reserved.
//

#import "FPReplay.h"

@implementation FPReplayFrame

- (id)initFromGame:(FPGame *)game
{
    self = [super init];
    if (self)
    {
        NSMutableArray *gameObjects = game.gameObjects;
        
        replayItems = (FPReplayItem *)malloc(gameObjects.count * sizeof(FPReplayItem));
        
        for (int i = 0; i < gameObjects.count; i++)
        {
            id<FPGameObject> gameObject = (id<FPGameObject>)[gameObjects objectAtIndex:i];
            replayItems[i].x = gameObject.x;
            replayItems[i].y = gameObject.y;
            replayItems[i].isVisible = gameObject.isVisible;
            if ([gameObject respondsToSelector:@selector(textureIndex)])
                replayItems[i].textureIndex = gameObject.textureIndex;
        }
        
        FPPlayer *player = (FPPlayer *)game.player;
        playerRotation = player.rotation;
        playerAlpha = player.alpha;
        playerSpeedUpCounter = player.speedUpCounter;
        
        backgroundOffset = game.backgroundOffset;
    }
    return self;
}

- (void)dealloc
{
    free(replayItems);
    [super dealloc];
}

- (void)setToGame:(FPGame *)game
{
    NSMutableArray *gameObjects = game.gameObjects;

    for (int i = 0; i < gameObjects.count; i++)
    {
        id<FPGameObject> gameObject = (id<FPGameObject>)[gameObjects objectAtIndex:i];
        float moveX = replayItems[i].x - gameObject.x;
        float moveY = replayItems[i].y - gameObject.y;
        [gameObject moveWithX:moveX y:moveY];
        [gameObject setIsVisible:replayItems[i].isVisible];
        if ([gameObject respondsToSelector:@selector(setTextureIndex:)])
            gameObject.textureIndex = replayItems[i].textureIndex;
    }
    
    FPPlayer *player = (FPPlayer *)game.player;
    player.rotation = playerRotation;
    player.alpha = playerAlpha;
    player.speedUpCounter = playerSpeedUpCounter;
    
    game.backgroundOffset = backgroundOffset;
}

@end

@implementation FPReplay

- (id)init
{
    self = [super init];
    if (self)
    {
        frames = [[NSMutableArray alloc] init];
        currentFrameIndex = 0;
    }
    return self;
}

- (void)dealloc
{
    [frames release];
    [super dealloc];
}

- (void)addFrameFromGame:(FPGame *)game
{
    FPReplayFrame *frame = [[FPReplayFrame alloc] initFromGame:game];
    [frames addObject:frame];
    [frame release];
}

- (BOOL)playFrameToGame:(FPGame *)game
{
    if (currentFrameIndex < frames.count)
    {
        FPReplayFrame *frame = (FPReplayFrame *)[frames objectAtIndex:currentFrameIndex];
        [frame setToGame:game];
        currentFrameIndex++;
        return YES;
    }
    return NO;
}

- (void)resetReplay
{
    currentFrameIndex = 0;
}

@end
